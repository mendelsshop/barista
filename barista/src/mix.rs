use crate::{
    config::{BlendConfig, Config},
    utils::{FindFileError, TomlOpenError},
};

use std::{
    fs,
    io::{self, Write},
};

#[derive(Debug, thiserror::Error)]
pub enum ConfigWriteError {
    #[error("{0}")]
    FindFile(FindFileError),
    #[error("error opening file {1}:  {0}")]
    FileOpen(io::Error, String),
    #[error("error writing file {1}:  {0}")]
    FileWrite(io::Error, String),
    #[error("error writing toml to file {1}:  {0}")]
    TomlWrite(toml::ser::Error, String),
    #[error("{0}")]
    TomlRead(TomlOpenError),
}

pub fn add_dependency(name: &str, blend: BlendConfig) -> Result<(), ConfigWriteError> {
    let config_file = Config::find_config().map_err(ConfigWriteError::FindFile)?;
    let mut config = Config::open_config(&config_file).map_err(ConfigWriteError::TomlRead)?;
    config.add_blend(name.to_string(), blend);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(config_file.clone())
        .map_err(|error| ConfigWriteError::FileOpen(error, config_file.display().to_string()))?; // map to FileOpen

    writeln!(
        file,
        "{}",
        toml::to_string(&config).map_err(|error| ConfigWriteError::TomlWrite(
            error,
            config_file.display().to_string()
        ))?
    ) // map to TomlWrite
    .map_err(|error| ConfigWriteError::FileWrite(error, config_file.display().to_string()))
    // map to FileWrite
}
