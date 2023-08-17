use std::{fs, io::Read, path::PathBuf};

use serde::Deserialize;

/// Tries to find a config file in the current directory or any of its ancestors
pub fn find_file(file_name: &str) -> Option<PathBuf> {
    std::env::current_dir().ok()?.ancestors().find_map(|dir| {
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
}

pub fn open_toml<T: for<'a> Deserialize<'a>>(path: PathBuf) -> Option<T> {
    match fs::File::open(path) {
        Ok(mut main) => {
            let mut buf = String::new();
            #[allow(clippy::redundant_pattern_matching)]
            // we might some day return error here (instead of option)
            if let Err(e) = main.read_to_string(&mut buf) {
                println!("Error reading file {e}");
                None
            } else {
                match toml::from_str(&buf) {
                    Ok(it) => it,
                    Err(e) => {
                        println!("Error reading file {e}");
                        return None;
                    }
                }
            }
        }
        Err(_) => None,
    }
}
