use reqwest::Client;
use serde::Deserialize;
use tokio::runtime::{Builder, Runtime};

use crate::ListType;

const BASE_URL: &'static str = "https://api.foojay.io/disco/v3.0/";

pub struct RequestBuilder {
    url: String,
    action: Box<dyn Fn(reqwest::RequestBuilder, &Runtime)>,
    runtime: Runtime,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            url: BASE_URL.to_owned(),
            action: Box::new(|_, _| {}),
            runtime: Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap(),
        }
    }

    pub fn list(mut self, list: ListType) -> Self {
        self.url += "distributions";
        if let ListType::Versions { distribution } = list {
            self.url = format!("{}/{}?latest_per_update=true", self.url, distribution);
            let execute = |req, runtime: &Runtime| {
                runtime
                    .block_on(runtime.spawn(get_runtime_versions(req)))
                    .unwrap();
            };
            self.action = Box::new(execute);
        } else {
            self.url += "?include_versions=false&include_synonyms=false";
            let execute = |req, runtime: &Runtime| {
                runtime
                    .block_on(runtime.spawn(get_distributions(req)))
                    .unwrap();
            };
            self.action = Box::new(execute);
        }
        self
    }

    pub fn execute(self) {
        let client = Client::new();
        (self.action)(client.get(self.url), &self.runtime);
    }
}

async fn get_runtime_versions(req: reqwest::RequestBuilder) {
    let json: RuntimeVersions = req
        .send()
        .await
        .expect("couldn't send request")
        .json()
        .await
        .unwrap();
    for version in &json.result[0].versions {
        println!("{version}")
    }
}
async fn get_distributions(req: reqwest::RequestBuilder) {
    let json: Distributions = req
        .send()
        .await
        .expect("couldn't send request")
        .json()
        .await
        .unwrap();
    for distribution in json.result {
        println!("{}", distribution.api_parameter)
    }
}

#[derive(Deserialize, serde::Serialize, Debug)]
pub struct RuntimeVersions {
    result: Vec<RuntimeVersionsResult>,
}
#[derive(Deserialize, serde::Serialize, Debug)]
pub struct RuntimeVersionsResult {
    versions: Vec<String>,
}

#[derive(Deserialize, serde::Serialize, Debug)]
pub struct Distributions {
    result: Vec<DistributionResult>,
}

#[derive(Deserialize, serde::Serialize, Debug)]
pub struct DistributionResult {
    api_parameter: String,
}
