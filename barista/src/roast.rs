// javac -cp lib/* src/Main.java
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

// // javac -c lib/* main & java -c lib/* main
use javaup::config;

use crate::{config::Config, utils::unless_exists};

pub fn roast(bin: Option<String>) {
    let config = Config::find_and_open_config().unwrap();
    config.fetch();
    let binding = crate::config::get_root_path().unwrap();
    let root = binding.display();
    let java_config = config::config_file();
    let mut java_bin = config::jdkdir();
    java_bin.push(java_config.default_jdk.clone().unwrap().distribution);
    java_bin.push(java_config.default_jdk.unwrap().version);
    java_bin.push("bin");
    #[cfg(target_os = "windows")]
    java_bin.push("javac.exe");
    #[cfg(not(target_os = "windows"))]
    java_bin.push("javac");
    let binding = bin
        .map(|bin| config.find_bin(bin).unwrap())
        .unwrap_or(PathBuf::from("Main.java"));
    let bin_path = binding.display();
    let mut binding = Command::new(java_bin);
    unless_exists(format!("{root}/src/{bin_path}"), ||{panic!("not bin target found")});
    let javac_ex = binding
        .arg("-cp")
        .arg(format!("{root}/lib/*",))
        .arg("--source-path")
        .arg(format!("{root}/src"))
        .arg(format!("{root}/src/{bin_path}"))
        .arg("-d")
        .arg(format!("{root}/bin"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    javac_ex.status().unwrap();
}
