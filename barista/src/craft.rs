//! craft.rs handles creating new brews and initializing non-barista projects as brews

const PSVM: &str = r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}"#;

const GITIGNORE: &str = "/lib
/bin";

// TOOD: should probably wrap around io::Result, so the user doesn't just get creation of x file/dir failed
// but rather creation of root directory failed ...

use std::{
    fs::{self, File},
    io::{self, Write},
};

use crate::config::Config;

pub fn create_new_brew(name: &str) -> io::Result<()> {
    fs::create_dir(name)?;
    init_brew(name, name)
}

pub fn init_brew(path: &str, name: &str) -> io::Result<()> {
    fs::create_dir(format!("{}/src", path))?;

    // TODO: this the lib directory should be created at compile time
    fs::create_dir(format!("{}/lib", path))?;

    init_main(path)?;
    init_gitignore(path)?;
    init_config(path, name)
}

fn init_file(path: &str, contents: &str) -> io::Result<()> {
    writeln!(
        File::options().write(true).create(true).open(path)?,
        "{}",
        contents
    )
}
fn init_config(path: &str, name: &str) -> io::Result<()> {
    init_file(
        &format!("{path}/brew.toml"),
        // TODO: could be slightly better error message
        &toml::to_string(&Config::new(name.to_string()))
            .expect("default brew.toml should not fail toml format [please file a bug]"),
    )
}

// TODO: when initializing non-barista packages we should account for Main.java being already created
fn init_main(path: &str) -> io::Result<()> {
    init_file(&format!("{path}/src/Main.java"), PSVM)
}

fn init_gitignore(path: &str) -> io::Result<()> {
    init_file(&format!("{path}/.gitignore"), GITIGNORE)
}
