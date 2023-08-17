use std::{fs::File, io::Write, str::FromStr};

use crate::config::{BlendConfig, Config};
use async_recursion::async_recursion;
use lenient_semver::Version;
use reqwest::Client;
use semver::{BuildMetadata, Prerelease};
use serde::Deserialize;
use tokio::runtime::Builder;

impl Config {
    pub fn fetch(&self) {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let mut dep_handles = Vec::with_capacity(self.blends().len());
        for (dep_name, dep_info) in self.blends().clone() {
            dep_handles.push(runtime.spawn(dep_info.fetch_maven(dep_name)))
        }
        for handle in dep_handles {
            // The `spawn` method returns a `JoinHandle`. A `JoinHandle` is
            // a future, so we can wait for it using `block_on`.
            runtime.block_on(handle).unwrap();
        }
    }
}
impl BlendConfig {
    async fn fetch_maven(self, name: String) {
        let client = Client::new();
        if let Some(maven_author) = self.author() {
            let req_url = format!(
                "https://repo1.maven.org/maven2/{}/{}/",
                maven_author.replace(".", "/"),
                name
            );
            let dep_info_url = req_url.clone() + "maven-metadata.xml";
            let text = client
                .get(dep_info_url)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            let dep_info_xml = quick_xml::de::from_str::<Metadata>(&text).unwrap();
            let version = self.find_best_version(dep_info_xml).expect(&format!(
                "couldn't find resolve version for {name} with version {}",
                self.version()
            ));
            finish_download_dep(name, version.0, req_url, client).await;
        } else {
            panic!()
        }
    }

    fn find_best_version<'a>(
        &self,
        dep_info_xml: Metadata<'a>,
    ) -> Option<(&'a str, semver::Version)> {
        dep_info_xml
            .versioning
            .versions
            .version
            .iter()
            .filter_map(|s| (Version::parse(*s).ok().map(|v| (*s, v))))
            .map(|(s, v)| (s, to_version(v)))
            .filter(|(_, version)| self.version().matches(version))
            .max()
    }
}
#[async_recursion]
async fn finish_download_dep(name: String, version: &str, req_url: String, client: Client) {
    let (dep_url, dep_url_info, dep_path) = {
        let path = format!("{}-{}", name, version);
        let url_base = req_url + &version + "/" + &path;
        (
            url_base.clone() + ".jar",
            url_base + ".pom",
            format!("lib/{path}") + ".jar",
        )
    };
    let res = client
        .get(&dep_url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", dep_url)))
        .unwrap();
    let dep = res.bytes().await.unwrap();
    let mut file = File::create(&dep_path)
        .or(Err(format!("Failed to create file '{}'", dep_path)))
        .unwrap();
    file.write_all(&dep[..])
        .expect(&format!("couldn't write to file '{}'", dep_path));
    download_dep_dep(client, &dep_url_info).await;
}

// donwlads a dependencies dependencies, gets its own function, b/c we don't need to do version resolution
async fn download_dep_dep(client: Client, pom_url: &str) {
    let text = client
        .get(pom_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let dep_info_xml = quick_xml::de::from_str::<Project>(&text).unwrap();
    match dep_info_xml.dependencies {
        Some(deps) => {
            for dep in deps.dependency.into_iter() {
                let req_url = format!(
                    "https://repo1.maven.org/maven2/{}/{}/",
                    dep.group_id.replace(".", "/"),
                    dep.artifact_id,
                );
                finish_download_dep(dep.artifact_id, &dep.version, req_url, client.clone()).await;
            }
        }
        None => {}
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Metadata<'a> {
    // group_id: String,
    // artifact_id: String,
    #[serde(borrow)]
    versioning: Versioning<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Versioning<'a> {
    // latest: Version<'a>,
    // release: Version<'a>,
    #[serde(borrow)]
    versions: Versions<'a>,
    // last_updated: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Versions<'a> {
    #[serde(borrow)]
    // reason we use &str as opposed to version
    // is because when we convert to version we don't know
    // if version had .0 at or not
    version: Vec<&'a str>,
}

pub fn to_version<'a>(version: Version<'a>) -> semver::Version {
    semver::Version {
        major: version.major,
        minor: version.minor,
        patch: version.patch,
        pre: Prerelease::from_str(&version.pre.map(|s| s.to_string()).unwrap_or("".to_string()))
            .unwrap(),
        build: BuildMetadata::from_str(
            &version
                .build
                .clone()
                .map(|s| s.to_string())
                .unwrap_or("".to_string()),
        )
        .unwrap(),
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    dependencies: Option<Dependencies>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dependencies {
    dependency: Vec<Dependency>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    group_id: String,
    artifact_id: String,
    version: String,
}
