//! craft.rs handles creating new brews and initializing non-barista projects as brews

const PSVM: &str = r#"public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}"#;

use std::fs;
use std::io::Write;

use crate::config;

pub fn create_new_brew(name: &str) {
    if let Err(e) = fs::create_dir(name) {
        panic!("Error creating directory {name} in creation of new Brew {name}: {e}");
    }
    init_brew(name, name)
}

pub fn init_brew(path: &str, name: &str) {
    if let Err(e) = fs::create_dir(format!("{}/src", path)) {
        panic!("Error creating source directory in creation of new Brew {path}: {e}");
    }

    init_main(path);
    init_config(path, name);
}

fn init_config(path: &str, name: &str) {
    match fs::File::options()
        .write(true)
        .create(true)
        .open(format!("{path}/brew.toml"))
    {
        Ok(mut config_file) => {
            let config = config::Config::new(name.to_string());
            if let Err(e) = writeln!(config_file, "{}", toml::to_string(&config).unwrap()) {
                panic!(
                    "Error initializing brew.toml file could not write default config to file: {}",
                    e
                )
            }
        }
        Err(e) => {
            panic!(
                "Error initializing brew.toml file could not create brew.toml file: {}",
                e
            )
        }
    }
}

// TODO: when initializing non-barista packages we should account for Main.java being already created
fn init_main(path: &str) {
    match fs::File::options()
        .create(true)
        .write(true)
        .open(format!("{path}/src/Main.java"))
    {
        Ok(mut main) => {
            if let Err(e) = writeln!(main, "{}", PSVM) {
                panic!("Error initializing Main.java file could not write hello world implementation to file: {}", e)
            }
        }
        Err(e) => {
            panic!(
                "Error initializing Main.java file could not create Main.java file: {}",
                e
            )
        }
    }
}
