use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use javaup::config;

use crate::{config::Config, roast::roast};

pub fn brew(bin: Option<String>) {
    roast(bin.clone());
    let java_config = config::config_file();
    let mut java_bin = config::jdkdir();

    java_bin.push(java_config.default_jdk.clone().unwrap().distribution);
    java_bin.push(java_config.default_jdk.unwrap().version);
    java_bin.push("bin");
    #[cfg(target_os = "windows")]
    java_bin.push("java.exe");
    #[cfg(not(target_os = "windows"))]
    java_bin.push("java");
    let binding = crate::config::get_root_path().unwrap();
    let root = binding.display();
    let config = Config::find_and_open_config().unwrap();
    let mut binding = bin
        .map(|bin| config.find_bin(bin).unwrap())
        .unwrap_or(PathBuf::from("Main.java"));
    binding.set_extension("");
    let bin_path = binding.display();
    let mut binding = Command::new(java_bin);
    let binding = binding
        .arg("-cp")
        .arg(format!("{root}/lib/*.:{root}/bin"))
        .arg(bin_path.to_string())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    binding.output().unwrap();
}
