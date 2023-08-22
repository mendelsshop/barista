use crate::{
    config::{BlendConfig, Config},
    utils::{FindFileError, TomlOpenError},
};
use std::{
    fs,
    io::{self, Write},
};

#[derive(Debug)]
pub enum ConfigWriteError {
    FindFile(FindFileError),
    FileOpen(io::Error),
    FileWrite(io::Error),
    TomlWrite(toml::ser::Error),
    TomlRead(TomlOpenError),
}

pub fn add_dependency(name: &str, blend: BlendConfig) -> anyhow::Result<()> {
    let config_file = Config::find_config()?;
    let mut config = Config::open_config(&config_file)?;
    config.add_blend(name.to_string(), blend);

    let mut file = fs::OpenOptions::new().write(true).open(config_file)?; // map to FileOpen

    Ok(
        writeln!(file, "{}", toml::to_string(&config)?) // map to TomlWrite
    ?,
    ) // map to FileWrite
}
