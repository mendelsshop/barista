use std::{collections::HashMap, path::PathBuf};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::utils::{find_file, open_toml, FindFileError, TomlOpenError};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    brew: BrewConfig,
    blends: HashMap<String, BlendConfig>,
    bin: Option<Vec<Bin>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Bin {
    name: String,
    path: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BrewConfig {
    name: String,
    version: Version,
}

impl BrewConfig {
    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

fn default_version() -> VersionReq {
    VersionReq::STAR
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlendConfig {
    author: Option<String>,
    path: Option<String>,
    #[serde(default = "default_version")]
    version: VersionReq,
    url: Option<String>,
}

impl BlendConfig {
    pub fn new_maven(version: VersionReq, author: String) -> Self {
        Self {
            author: Some(author),
            path: None,
            version,
            url: None,
        }
    }

    pub fn new_git(version: VersionReq, url: String) -> Self {
        Self {
            author: None,
            path: None,
            version,
            url: Some(url),
        }
    }

    pub fn new_path(version: VersionReq, path: String) -> Self {
        Self {
            author: None,
            path: Some(path),
            version,
            url: None,
        }
    }

    pub fn author(&self) -> Option<&String> {
        self.author.as_ref()
    }

    pub fn version(&self) -> &VersionReq {
        &self.version
    }
}
#[derive(Debug)]
pub enum OpenConfigError {
    FindFileError(FindFileError),
    TomlOpenError(TomlOpenError),
}

impl Config {
    /// Create a new config with the given name
    pub fn new(name: String) -> Self {
        Self {
            brew: BrewConfig {
                name,
                version: Version::new(0, 1, 0),
            },
            blends: HashMap::new(),
            bin: None,
        }
    }

    pub fn open_config(path: &PathBuf) -> Result<Self, TomlOpenError> {
        open_toml(path)
    }

    pub fn find_and_open_config() -> Result<Self, OpenConfigError> {
        Self::find_config()
            .map_err(OpenConfigError::FindFileError) // map to FindFileError
            .and_then(|config_path| {
                Config::open_config(&config_path).map_err(OpenConfigError::TomlOpenError)
            }) // map to TomlOpenError
    }

    pub fn add_blend(&mut self, name: String, blend: BlendConfig) {
        self.blends.insert(name, blend);
    }

    pub fn find_config() -> Result<PathBuf, FindFileError> {
        find_file("Brew.toml")
    }

    pub fn blends(&self) -> &HashMap<String, BlendConfig> {
        &self.blends
    }

    pub fn brew(&self) -> &BrewConfig {
        &self.brew
    }
    pub fn find_bin(&self, bin_name: String) -> Result<PathBuf, BinNotFoundError> {
        self.bin
            .as_ref()
            .ok_or(BinNotFoundError::NoTargetsSpecefied)?
            .iter()
            .find_map(|bin| {
                (bin.name == bin_name).then(|| {
                    bin.path.as_ref().map_or_else(
                        || PathBuf::from_iter(["bin", &(bin.name.clone() + ".java")]),
                        PathBuf::from,
                    )
                })
            })
            .ok_or(BinNotFoundError::NoTargetsMatched(bin_name))
    }
}

#[derive(Debug)]
pub enum BinNotFoundError {
    NoTargetsSpecefied,
    NoTargetsMatched(String),
}

#[derive(Debug)]
pub enum FindRootError {
    FindFileError(FindFileError),
    FoundConfigWithoutRoot,
}

pub fn get_root_path() -> Result<PathBuf, FindRootError> {
    find_file("Brew.toml")
        .map_err(FindRootError::FindFileError)?
        .parent()
        .map(|path| path.to_path_buf())
        .ok_or(FindRootError::FoundConfigWithoutRoot)
}
