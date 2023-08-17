use crate::{ListType, ToolChain};
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::Deserialize;
use std::env::consts::ARCH;
use std::{
    env::{self, consts::OS},
    fs::File,
    io::Write,
    path::PathBuf,
};
use tar::Archive;
use tokio::runtime::{Builder, Runtime};

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

    pub fn install(mut self, jdk: ToolChain) -> Self {
        self.url = format!(
            "{}packages/jdks?version={}&distribution={}&architecture={ARCH}&archive_type=tar.gz&operating_system{OS}&lib_c_type={}",
            self.url, jdk.version, jdk.distribution, lib_c_name()
        );
        let execute = |req, runtime: &Runtime| {
            runtime.block_on(runtime.spawn(install_jdk(req))).unwrap();
        };
        self.action = Box::new(execute);
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

pub fn jdkdir() -> PathBuf {
    let mut path_buf = dirs::data_dir().unwrap();
    path_buf.push("javaup/jdks/");
    if matches!(path_buf.try_exists(), Err(_) | Ok(false)) {
        std::fs::create_dir_all(&path_buf);
    }
    path_buf
}

async fn install_jdk(req: reqwest::RequestBuilder) {
    let client = Client::new();
    dirs::data_dir().unwrap().push("javaup");
    let json: JDKInfos = req
        .send()
        .await
        .expect("couldn't send request")
        .json()
        .await
        .unwrap();
    let mut jdkdir = jdkdir();
    jdkdir.push(&json.result[0].filename);
    let mut file = File::create(&jdkdir).unwrap();
    let res = client
        .get(&json.result[0].links.pkg_download_redirect)
        .send()
        .await
        .or(Err(format!(
            "Failed to GET from '{}'",
            &json.result[0].links.pkg_download_redirect
        )))
        .unwrap();
    let dep = res.bytes().await.unwrap();
    file.write_all(&dep[..])
        .unwrap_or_else(|_| panic!("couldn't write to file "));
    let decompress_stream = GzDecoder::new(File::open(&jdkdir).unwrap());
    let mut a = Archive::new(decompress_stream);
    let dst = jdkdir.to_str().unwrap().strip_suffix(".tar.gz").unwrap();
    a.unpack(dst).unwrap();
    println!("{dst}")
}

pub const fn lib_c_name() -> &'static str {
    match platforms::TARGET_ENV {
        Some(env) => match env {
            platforms::target::Env::Gnu => "glibc",
            platforms::target::Env::Msvc => "c_std_lib",
            platforms::target::Env::Musl => "musl",
            _ => "libc",
        },
        _ => "libc",
    }
}

#[derive(Deserialize, Debug)]
pub struct RuntimeVersions {
    result: Vec<RuntimeVersionsResult>,
}
#[derive(Deserialize, Debug)]
pub struct RuntimeVersionsResult {
    versions: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Distributions {
    result: Vec<DistributionResult>,
}

#[derive(Deserialize, Debug)]
pub struct DistributionResult {
    api_parameter: String,
}

#[derive(Deserialize, Debug)]
pub struct JDKInfos {
    result: Vec<JDKInfo>,
}

#[derive(Deserialize, Debug)]
pub struct JDKInfo {
    filename: String,
    links: JDKDownload,
}

#[derive(Deserialize, Debug)]
pub struct JDKDownload {
    pkg_download_redirect: String,
}
