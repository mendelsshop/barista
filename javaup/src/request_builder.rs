use crate::{ListType, ToolChain};
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::Deserialize;
use std::{
    env::consts::ARCH,
    fs,
    io::{self, Read},
    path::Path,
};
use std::{env::consts::OS, path::PathBuf};
use tar::Archive;
use tokio::runtime::{Builder, Runtime};

const BASE_URL: &str = "https://api.foojay.io/disco/v3.0/";

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
    add_to_path(root_dir(), "jdks")
}

pub fn tmpdir() -> PathBuf {
    add_to_path(root_dir(), "tmp")
}
pub fn root_dir() -> PathBuf {
    add_to_path(
        dirs::home_dir().expect("no home directory find javaup directory, javaup cannot continue"),
        ".javaup",
    )
}

pub fn init_dirs() {
    let root = root_dir();
    fs::create_dir(root).expect(
        "failed to create .javaup (root) directory, initialzaition of javaup cannot continue",
    );
    fs::create_dir(jdkdir())
        .expect("failed to create jdks directory, initialzaition of javaup cannot continue");
    fs::create_dir(tmpdir())
        .expect("failed to create jdks directory, initialzaition of javaup cannot continue");
}

fn add_to_path(mut dir: PathBuf, path: &str) -> PathBuf {
    dir.push(path);
    dir
}

async fn install_jdk(req: reqwest::RequestBuilder) {
    let client = Client::new();
    let json: JDKInfos = req
        .send()
        .await
        .expect("couldn't send request")
        .json()
        .await
        .unwrap();
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
    let decompress_stream = GzDecoder::new(&dep[..]);
    let a = Archive::new(decompress_stream);
    let dist_dir = add_to_path(jdkdir(), &json.result[0].distribution);
    unless_exists(&dist_dir, || {
        fs::create_dir(&dist_dir)
    .expect("failed to create jdk distribution directory, initialzaition of new jdk cannot continue")
    });
    let path = add_to_path(dist_dir, &json.result[0].jdk_version.to_string());
    fs::create_dir(&path).expect(
        "failed to create jdk versiom directory, initialzaition of new jdk cannot continue",
    );
    unpack_sans_parent(a, path)
        .expect("failed to unpack jdk, initialzaition of new jdk cannot continue");
}

pub fn unless_exists(path: &Path, f: impl Fn()) {
    if matches!(path.try_exists(), Err(_) | Ok(false)) {
        f()
    }
}
// from https://users.rust-lang.org/t/moving-and-renaming-directory/44742/7
pub fn unpack_sans_parent<R, P>(mut archive: Archive<R>, dst: P) -> Result<(), io::Error>
where
    R: Read,
    P: AsRef<Path>,
{
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path: PathBuf = entry
            .path()?
            .components()
            .skip(1) // strip top-level directory
            .filter(|c| matches!(c, std::path::Component::Normal(_))) // prevent traversal attacks
            .collect();
        entry.unpack(dst.as_ref().join(path))?;
    }
    Ok(())
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
    jdk_version: i32,
    distribution: String,
    links: JDKDownload,
}

#[derive(Deserialize, Debug)]
pub struct JDKDownload {
    pkg_download_redirect: String,
}
