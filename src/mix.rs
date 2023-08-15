use crate::config::{BlendConfig, Config};
use std::fs;
use std::io::Write;

pub fn add_dependency(name: &str, blend: BlendConfig) {
    let config_file = Config::find_config();
    let mut config = config_file
        .clone()
        .map(Config::open_config)
        .flatten()
        .expect("Couldn't find configuration");
    config.add_blend(name.to_string(), blend);

    // its ok to unwrap the config as is already been verified to be something (not None) with `.expect("Couldn't find configuration");`
    match fs::OpenOptions::new()
        .write(true)
        .open(config_file.unwrap())
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", toml::to_string(&config).unwrap()) {
                panic!(
                    "Error initializing brew.toml file could not write default config to file: {}",
                    e
                )
            }
        }
        Err(e) => {
            panic!("Error config file moved since last read in middle of adding Blend {name} {e}")
        }
    }
}
