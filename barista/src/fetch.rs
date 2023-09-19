use std::{
    fs::{File, self},
    io::Write,
    path::{PathBuf, Path},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    config::{BlendConfig, Config},
    lock::{LockFile, Package}, utils::unless_exists,
};
use async_recursion::async_recursion;
use lenient_semver::Version;
use reqwest::Client;
use semver::{BuildMetadata, Prerelease};
use serde::Deserialize;
use tokio::runtime::Builder;

impl Config {
    pub fn fetch(&self) {
        let binding = crate::config::get_root_path().unwrap();
        let root = binding.display();
        unless_exists(Path::new(&format!("{root}/lib")), || {
            fs::create_dir_all(format!("{root}/lib"))
                .expect("Failed to create Brew Library directory (lib) when building")
        });
        let lock_file = LockFile::new(self.brew().name().to_owned(), self.brew().version().clone());
        let locked_lock_file = Arc::new(Mutex::new(lock_file));
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let mut dep_handles = Vec::with_capacity(self.blends().len());
        for (dep_name, dep_info) in self.blends().clone() {
            dep_handles
                .push(runtime.spawn(dep_info.fetch_maven(dep_name, locked_lock_file.clone())));
        }
        for handle in dep_handles {
            // The `spawn` method returns a `JoinHandle`. A `JoinHandle` is
            // a future, so we can wait for it using `block_on`.
            runtime.block_on(handle).unwrap();
        }
        let lock_file = locked_lock_file.lock().unwrap();
        let lock_file_path = get_lock_path();
        let mut lock_file_file = File::create(lock_file_path).unwrap();
        writeln!(
            lock_file_file,
            "{}",
            toml::ser::to_string(&*lock_file).unwrap()
        )
        .expect("Could not write new Barista.lock after fetching dependencies");
    }
}
impl BlendConfig {
    async fn fetch_maven(self, name: String, locked_lock_file: Arc<Mutex<LockFile>>) {
        let client = Client::new();
        if let Some(maven_author) = self.author() {
            let req_url = format!(
                "https://repo1.maven.org/maven2/{}/{}/",
                maven_author.replace('.', "/"),
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
            let version = self.find_best_version(dep_info_xml).unwrap_or_else(|| {
                panic!(
                    "couldn't find resolve version for {name} with version {}",
                    self.version()
                )
            });

            let blend_dep = Package::new(
                name.clone(),
                version.1.to_string(),
                maven_author.clone(),
                // TODO: get the user frinedly url for this dependency too
                req_url.clone(),
                None,
                None,
            );

            finish_download_dep(
                name,
                version.0,
                req_url,
                client,
                locked_lock_file,
                blend_dep,
            )
            .await;
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
            .filter_map(|s| (Version::parse(s).ok().map(|v| (*s, v))))
            .map(|(s, v)| (s, to_version(v)))
            .filter(|(_, version)| self.version().matches(version))
            .max()
    }
}
#[async_recursion]
async fn finish_download_dep(
    name: String,
    version: &str,
    req_url: String,
    client: Client,
    locked_lock_file: Arc<Mutex<LockFile>>,
    package: Package,
) {
    let (dep_url, dep_url_info, dep_path) = {
        let path = format!("{}-{}", name, version);
        let url_base = req_url + version + "/" + &path;
        (
            url_base.clone() + ".jar",
            url_base + ".pom",
            // TODO: this doesn't account for the case where barista isn't ran from the project root dir  (where lib doesnt exist)
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
        .unwrap_or_else(|_| panic!("couldn't write to file '{}'", dep_path));
    download_dep_dep(client, &dep_url_info, locked_lock_file, package).await;
}

// donwlads a dependencies dependencies, gets its own function, b/c we don't need to do version resolution
async fn download_dep_dep(
    client: Client,
    pom_url: &str,
    locked_lock_file: Arc<Mutex<LockFile>>,
    mut package: Package,
) {
    let text = client
        .get(pom_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let dep_info_xml = quick_xml::de::from_str::<Project>(&text).expect(pom_url);
    if let Some(deps) = dep_info_xml.dependencies {
        let filterdeps = deps.dependency.into_iter().filter(|dep| {
            (dep.scope.content == MavenDependencyScopeType::Compile
                || dep.scope.content == MavenDependencyScopeType::Runtime)
                && !dep.optional
        });
        for dep in filterdeps.clone() {
            let req_url = format!(
                "https://repo1.maven.org/maven2/{}/{}/",
                dep.group_id.replace('.', "/"),
                dep.artifact_id,
            );
            let blend_dep = Package::new(
                dep.artifact_id.clone(),
                dep.version.clone(),
                dep.group_id.clone(),
                req_url.clone(),
                None,
                None,
            );
            finish_download_dep(
                dep.artifact_id,
                &dep.version,
                req_url,
                client.clone(),
                locked_lock_file.clone(),
                blend_dep,
            )
            .await;
        }
        // get all parent dependencies as names
        let deps = filterdeps.map(|dep| dep.artifact_id).collect();
        package.set_dependencies(deps);
        write_package_to_lockfile(package, locked_lock_file);
    }
}

fn write_package_to_lockfile(packed: Package, locked_lock_file: Arc<Mutex<LockFile>>) {
    if let Ok(mut lock_file) = locked_lock_file.lock() {
        lock_file.push(packed);
    }
}

fn get_lock_path() -> PathBuf {
    let mut root = crate::config::get_root_path().unwrap();
    root.push("Brew.lock");
    root
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

pub fn to_version(version: Version<'_>) -> semver::Version {
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

/// full spec found https://maven.apache.org/xsd/maven-4.0.0.xsd
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    dependencies: Option<Dependencies>,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dependencies {
    #[serde(default)]
    dependency: Vec<Dependency>,
}
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]

pub struct Dependency {
    group_id: String,
    artifact_id: String,
    version: String,
    #[serde(default)]
    scope: MavenDependencyScope,
    #[serde(default)]
    optional: bool,
}

// We need to have this wrapper around MavenDependencyScopeType see https://docs.rs/quick-xml/latest/quick_xml/de/index.html#enumunit-variants-as-a-text
#[derive(Deserialize, Debug, Default, Clone)]
pub struct MavenDependencyScope {
    #[serde(rename = "$text")]
    content: MavenDependencyScopeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MavenDependencyScopeType {
    #[default]
    Compile,
    Runtime,
    Test,
    Provided,
}
