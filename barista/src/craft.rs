//! craft.rs handles creating new brews and initializing non-barista projects as brews

const PSVM: &str = r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}"#;

const GITIGNORE: &str = "/lib
/bin
/doc";

// TOOD: should probably wrap around io::Result, so the user doesn't just get creation of x file/dir failed
// but rather creation of root directory failed ...

use std::{
    fs::{self, File},
    io::{self, Write},
};

#[derive(Debug, thiserror::Error)]
pub enum BrewCreationError {
    #[error("error creating {2:?} directory: {0}, path: {1}")]
    DirCreation(io::Error, String, DirectoryType),
    #[error("error creating {2:?} file : {0}, path: {1}")]
    FileOpen(io::Error, String, FileType),
    #[error("error creating {2:?} file : {0}, path: {1}")]
    FileWrite(io::Error, String, FileType),
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Gitignore,
    Main,
    Config,
}
#[derive(Debug)]
pub enum DirectoryType {
    Brew,
    Source,
}

use crate::config::Config;

pub fn create_new_brew(name: &str) -> Result<(), BrewCreationError> {
    fs::create_dir(name).map_err(|error| {
        BrewCreationError::DirCreation(error, name.to_string(), DirectoryType::Brew)
    })?;
    init_brew(name, name)
}

pub fn init_brew(path: &str, name: &str) -> Result<(), BrewCreationError> {
    fs::create_dir(format!("{}/src", path)).map_err(|error| {
        BrewCreationError::DirCreation(error, name.to_string(), DirectoryType::Source)
    })?;

    init_main(path)?;
    init_gitignore(path)?;
    init_config(path, name)
}

fn init_file(path: &str, contents: &str, ft: FileType) -> Result<(), BrewCreationError> {
    writeln!(
        File::options()
            .write(true)
            .create(true)
            .open(path)
            .map_err(|error| BrewCreationError::FileOpen(error, path.to_string(), ft))?,
        "{}",
        contents
    )
    .map_err(|error| BrewCreationError::FileWrite(error, path.to_string(), ft))
}
fn init_config(path: &str, name: &str) -> Result<(), BrewCreationError> {
    init_file(
        &format!("{path}/Brew.toml"),
        // TODO: could be slightly better error message
        &toml::to_string(&Config::new(name.to_string()))
            .expect("default Brew.toml should not fail toml format [please file a bug]"),
        FileType::Config,
    )
}

// TODO: when initializing non-barista packages we should account for Main.java being already created
fn init_main(path: &str) -> Result<(), BrewCreationError> {
    init_file(&format!("{path}/src/Main.java"), PSVM, FileType::Main)
}

fn init_gitignore(path: &str) -> Result<(), BrewCreationError> {
    init_file(
        &format!("{path}/.gitignore"),
        GITIGNORE,
        FileType::Gitignore,
    )
}
