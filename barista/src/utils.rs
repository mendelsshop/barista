use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum FindFileError {
    #[error("io error occured")]
    IO(io::Error),
    #[error("could not find {0}")]
    FileNotFound(String),
}

/// Tries to find a config file in the current directory or any of its ancestors
pub fn find_file(file_name: &str) -> Result<PathBuf, FindFileError> {
    std::env::current_dir()
        .map_err(FindFileError::IO)?
        .ancestors()
        .find_map(|dir| {
            if let Ok(mut dir) = dir.read_dir() {
                dir.find_map(|file| {
                    if let Ok(file) = file {
                        if let Ok(file_type) = file.file_type() {
                            if file_type.is_file() && file.path().ends_with(file_name) {
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
        .ok_or(FindFileError::FileNotFound(file_name.to_string()))
}
#[derive(Debug, thiserror::Error)]
pub enum TomlOpenError {
    #[error("error could not open file: {0}, path: {1}")]
    Open(io::Error, String),
    #[error("error could not read file: {0}, path: {1}")]
    Read(io::Error, String),
    #[error("error could not read file as toml {0}, path: {1}")]
    Toml(toml::de::Error, String),
}

pub fn open_toml<T: for<'a> Deserialize<'a>>(path: &PathBuf) -> Result<T, TomlOpenError> {
    let mut file =
        File::open(path).map_err(|error| TomlOpenError::Open(error, path.display().to_string()))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|error| TomlOpenError::Read(error, path.display().to_string()))?;
    toml::from_str(&buf).map_err(|error| TomlOpenError::Toml(error, path.display().to_string()))
}
