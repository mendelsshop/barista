use std::{collections::HashMap, path::PathBuf, str::FromStr};

use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::utils::{find_file, open_toml};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    brew: BrewConfig,
    blends: HashMap<String, BlendConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BrewConfig {
    name: String,
    version: VersionReq
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
    pub fn new_maven(version: String, author: String) -> Self {
        Self {
            author: Some(author),
            path: None,
            version:  VersionReq::parse(&version).unwrap(),
            url: None,
        }
    }

    pub fn new_git(version: String, url: String) -> Self {
        Self {
            author: None,
            path: None,
            version:  VersionReq::parse(&version).unwrap(),
            url: Some(url),
        }
    }

    pub fn new_path(version: String, path: String) -> Self {
        Self {
            author: None,
            path: Some(path),
            version:  VersionReq::parse(&version).unwrap(),
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

impl Config {
    /// Create a new config with the given name
    pub fn new(name: String) -> Self {
        Self {
            brew: BrewConfig {
                name,
                version:  VersionReq::from_str("0.1.0").unwrap()
            },
            blends: HashMap::new(),
        }
    }

    pub fn open_config(path: PathBuf) -> Option<Self> {
        open_toml(path)
    }

    pub fn find_and_open_config() -> Option<Self> {
        Self::find_config().and_then(Config::open_config)
    }

    pub fn add_blend(&mut self, name: String, blend: BlendConfig) {
        self.blends.insert(name, blend);
    }

    pub fn find_config() -> Option<PathBuf> {
        find_file("brew.toml")
    }

    pub fn blends(&self) -> &HashMap<String, BlendConfig> {
        &self.blends
    }
}
