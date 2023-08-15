use std::{collections::HashMap, fs, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    brew: BrewConfig,
    blends: HashMap<String, BlendConfig>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BrewConfig {
    name: String,
    version: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlendConfig {
    author: Option<String>,
    path: Option<String>,
    version: Option<String>,
    url: Option<String>,
}

impl BlendConfig {
    pub fn new_maven(version: String, author: String) -> Self {
        Self {
            author: Some(author),
            path: None,
            version: Some(version),
            url: None,
        }
    }

    pub fn new_git(version: String, url: String) -> Self {
        Self {
            author: None,
            path: None,
            version: Some(version),
            url: Some(url),
        }
    }

    pub fn new_path(version: String, path: String) -> Self {
        Self {
            author: None,
            path: Some(path),
            version: Some(version),
            url: None,
        }
    }
}

impl Config {
    /// Create a new config with the given name
    pub fn new(name: String) -> Self {
        Self {
            brew: BrewConfig {
                name,
                version: "0.1.0".to_string(),
            },
            blends: HashMap::new(),
        }
    }

    /// Tries to find a config file in the current directory or any of its ancestors
    pub fn find_config() -> Option<PathBuf> {
        std::env::current_dir().ok()?.ancestors().find_map(|dir| {
            if let Ok(mut dir) = dir.read_dir() {
                dir.find_map(|file| {
                    if let Ok(file) = file {
                        if let Ok(file_type) = file.file_type() {
                            if file_type.is_file() && file.path().ends_with("brew.toml") {
                                return Some(file.path());
                            }
                        }
                    }
                    None
                })
            } else {
                None
            }
        })
    }

    pub fn open_config(path: PathBuf) -> Option<Self> {
        match fs::File::open(path) {
            Ok(mut main) => {
                let mut buf = String::new();
                #[allow(clippy::redundant_pattern_matching)]
                // we might some day return error here (instead of option)
                if let Err(_) = main.read_to_string(&mut buf) {
                    None
                } else {
                    toml::from_str(&buf).ok()?
                }
            }
            Err(_) => None,
        }
    }

    pub fn find_and_open_config() -> Option<Self> {
        Config::find_config().and_then(Config::open_config)
    }

    pub fn add_blend(&mut self, name: String, blend: BlendConfig) {
        self.blends.insert(name, blend);
    }
}
