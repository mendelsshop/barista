

use std::str::FromStr;

use reqwest::Client;
use semver::{VersionReq, Comparator, Prerelease};
use serde::{Deserialize, Deserializer};
use tokio::runtime::Builder;
use crate::config::{Config, BlendConfig};
use lenient_semver::Version;

impl Config {
    pub fn fetch(&self) {
        let runtime =Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let mut dep_handles = Vec::with_capacity(self.blends().len());
        for (dep_name, dep_info) in self.blends().clone() {
            
            
                dep_handles.push(runtime.spawn(dep_info.fetch_maven(dep_name)))
            

        }
        for handle in  dep_handles {
            // The `spawn` method returns a `JoinHandle`. A `JoinHandle` is
            // a future, so we can wait for it using `block_on`.
            runtime.block_on(handle).unwrap();
        }
    }
    
}
impl BlendConfig {

async fn fetch_maven(self, name: String) {
    let client = Client::new();
    if let Some(maven_author)= self.author() {
    let req_url = format!("https://repo1.maven.org/maven2/{}/{}/", maven_author.replace(".", "/"), name);
    let dep_info_url= req_url + "maven-metadata.xml";
    let text = client.get(dep_info_url).send().await.unwrap().text().await.unwrap();
    let dep_info_xml = quick_xml::de::from_str::<metadata>(&text).unwrap();
    Into::<semver::Version>::into(self.version().clone()) <= dep_info_xml.versioning.latest;
    }
    else {
        panic!()
    }
}
}

#[derive(Deserialize , Debug)]
pub struct metadata<'a> {
    groupId: String,
    artifactId: String,
    #[serde(borrow)]
    versioning: Versioning<'a>,
}


#[derive(Deserialize, Debug)]
pub struct Versioning<'a> {
    latest: Version<'a>,
    release: Version<'a>,
    #[serde(borrow)]
    versions: Versions<'a>,
    lastUpdated: String
}


#[derive(Deserialize, Debug)]
pub struct Versions<'a> {
    #[serde(borrow)]
    version: Vec<Version<'a>>,
}

pub fn to_version<'a>(version: Version<'a>) -> VersionReq {
    VersionReq {
        comparators: vec![ Comparator { op: semver::Op::Caret, major: version.major, minor: Some(version.minor), patch: Some(version.patch), pre: Prerelease::from_str(&version.pre.unwrap().to_string()).unwrap(), }]
    }
}