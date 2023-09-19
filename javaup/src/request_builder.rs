use crate::{
    config::{add_to_path, config_file, jdkdir, unless_exists, write_config},
    ListType, ToolChain,
};
use flate2::read::GzDecoder;
use platforms::target::Env;
use reqwest::Client;
use serde::Deserialize;
use std::{
    env::consts::{ARCH, OS},
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};
use tar::Archive;
use tokio::runtime::{Builder, Runtime};

const BASE_URL: &str = "https://api.foojay.io/disco/v3.0/";

pub struct RequestBuilder {
    url: String,
    action: Box<dyn Fn(reqwest::RequestBuilder, &Runtime)>,
    runtime: Runtime,
}
impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
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
            // we inclue `&archive_type=tar` but there seems to be no jdk with that type of archaive
            "{}packages/jdks?version={}&distribution={}&architecture={ARCH}&archive_type=tar.gz&archive_type=tar&archive_type=zip&operating_system{OS}&lib_c_type={}&latest=overall&free_to_use_in_production=true",
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

async fn install_jdk(req: reqwest::RequestBuilder) {
    let client = Client::new();
    let json: JDKInfos = req
        .send()
        .await
        .expect("couldn't send request")
        .json()
        .await
        .unwrap();
    let jdkinfo = &json.result[0];
    let res = client
        .get(&jdkinfo.links.pkg_download_redirect)
        .send()
        .await
        .or(Err(format!(
            "Failed to GET from '{}'",
            &jdkinfo.links.pkg_download_redirect
        )))
        .unwrap();
    let dep = res.bytes().await.unwrap();
    let decompress_stream = GzDecoder::new(&dep[..]);
    let a = Archive::new(decompress_stream);
    let dist_dir = add_to_path(jdkdir(), &jdkinfo.distribution);
    unless_exists(&dist_dir, || {
        fs::create_dir(&dist_dir)
    .expect("failed to create jdk distribution directory, initialzaition of new jdk cannot continue")
    });

    let path = add_to_path(dist_dir, &jdkinfo.jdk_version.to_string());
    fs::create_dir(&path).expect(
        "failed to create jdk versiom directory, initialzaition of new jdk cannot continue",
    );
    // this seems to not work for macos
    // b/c macos jdk archieve (besisdes for zulu ones (so far))
    // have format when extracted root-dir (usually has jdk version in it)/Contents/Home
    // although there is a dylib in root/Contents/Macos
    // and plist in root/Content

    // for zulu on macos its root/(syslinks to all the needed things almost like linux) and root/innerroot/(same as other macos versions)

    // We also need to handle zips
    if jdkinfo.archive_type == "zips" {
        panic!("zips is not supported")
    }
    if jdkinfo.operating_system == SupportedOs::Macos && jdkinfo.distribution != "zulu"
        || jdkinfo.distribution == "zulu_prime"
    {
        unpack_sans_parent_macos(a, path)
            .expect("failed to unpack jdk, initialzaition of new jdk cannot continue");
    } else {
        unpack_sans_parent(a, path)
            .expect("failed to unpack jdk, initialzaition of new jdk cannot continue");
    }
    let mut config = config_file();
    if config.default_jdk.is_none() {
        config.default_jdk = Some(ToolChain {
            version: jdkinfo.jdk_version.to_string(),
            distribution: jdkinfo.distribution.to_string(),
        });
        write_config(&config)
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

pub fn unpack_sans_parent_macos<R, P>(mut archive: Archive<R>, dst: P) -> Result<(), io::Error>
where
    R: Read,
    P: AsRef<Path>,
{
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path: PathBuf = entry
            .path()?
            .components()
            .skip(3) // strip top-level directory
            .filter(|c| matches!(c, std::path::Component::Normal(_))) // prevent traversal attacks
            .collect();
        // if there are files that are 3 dirs deep they should be skipped (might not be best approach (as we might need thes filex main offender is a .plist))
        // another idea is to syslink everthing in roo like zulu
        if path.iter().count() == 0 {
            continue;
        }
        entry.unpack(dst.as_ref().join(path))?;
    }
    Ok(())
}

pub const fn lib_c_name() -> &'static str {
    match platforms::TARGET_ENV {
        Some(env) => match env {
            Env::Gnu => "glibc",
            Env::Msvc => "c_std_lib",
            Env::Musl => "musl",
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
    operating_system: SupportedOs,
    archive_type: String,
}

#[derive(Deserialize, Debug)]
pub struct JDKDownload {
    pkg_download_redirect: String,
}
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportedOs {
    Aix,
    AlpineLinux,
    Linux,
    LinuxMusl,
    Macos,
    Qnx,
    Solaris,
    Windows,
}
