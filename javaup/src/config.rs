use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::ToolChain;
#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub default_jdk: Option<ToolChain>,
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

pub fn config_file_path() -> PathBuf {
    add_to_path(root_dir(), "settings.toml")
}

pub fn config_file() -> Settings {
    let mut buf = String::new();
    fs::File::open(config_file_path())
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();
    toml::de::from_str(&buf).unwrap()
}

pub fn write_config(config: &Settings) {
    writeln!(
        fs::File::create(config_file_path()).unwrap(),
        "{}",
        toml::ser::to_string(&config).unwrap()
    )
    .unwrap();
}

pub fn init() {
    let root = root_dir();
    fs::create_dir(root).expect(
        "failed to create .javaup (root) directory, initialzaition of javaup cannot continue",
    );
    fs::create_dir(jdkdir())
        .expect("failed to create jdks directory, initialzaition of javaup cannot continue");
    fs::create_dir(tmpdir())
        .expect("failed to create jdks directory, initialzaition of javaup cannot continue");
    writeln!(fs::File::create(config_file_path()).expect("failed to create Settings.toml configuration file, initialzaition of javaup cannot continue"), "{}", toml::ser::to_string(&Settings {
        default_jdk: None,
    }).expect("failed to get default Settings.toml configuration file content, initalization of javaup cannot continue")).expect("failed to write default settings.toml configuration file content, initalization of javaup cannot continue");
}

pub fn add_to_path(mut dir: PathBuf, path: &str) -> PathBuf {
    dir.push(path);
    dir
}

pub fn unless_exists(path: &Path, f: impl Fn()) {
    if matches!(path.try_exists(), Err(_) | Ok(false)) {
        f()
    }
}
