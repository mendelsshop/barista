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
    #[error("could not find file")]
    FileNotFound,
}

/// Tries to find a config file in the current directory or any of its ancestors
pub fn find_file(file_name: &str) -> anyhow::Result<PathBuf> {
    std::env::current_dir()?
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
        .ok_or(FindFileError::FileNotFound.into())
}
#[derive(Debug)]
pub enum TomlOpenError {
    IO(io::Error),
    Toml(toml::de::Error),
}

pub fn open_toml<T: for<'a> Deserialize<'a>>(path: &PathBuf) -> anyhow::Result<T> {
    let mut file = File::open(path)?; // map to IO
    let mut buf = String::new();
    file.read_to_string(&mut buf)?; // map to IO
    Ok(toml::from_str(&buf)?) // map to Toml
}
