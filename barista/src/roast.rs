// javac -cp lib/* src/Main.java
use std::{
    fs,
    process::{Command, Stdio},
};

use anyhow::Context;
// // javac -c lib/* main & java -c lib/* main
use javaup::config;

use crate::config::Config;

pub fn roast() {
    Config::find_and_open_config().unwrap().fetch();
    // TODO: need better way obtaining this brews root directory
    let binding = Config::find_config().unwrap();
    let root = binding.parent().unwrap().display();
    let java_config = config::config_file();
    let mut java_bin = config::jdkdir();
    fs::create_dir(format!("{}/lib", root))
        .with_context(|| format!("Failed to create Brew Library (lib) when building"))
        .unwrap();
    java_bin.push(java_config.default_jdk.clone().unwrap().distribution);
    java_bin.push(java_config.default_jdk.unwrap().version);
    java_bin.push("bin");
    #[cfg(target_os = "windows")]
    java_bin.push("javac.exe");
    #[cfg(not(target_os = "windows"))]
    java_bin.push("javac");

    let mut binding = Command::new(java_bin);
    let javac_ex = binding
        .arg("-cp")
        .arg(format!("{root}/lib/*",))
        .arg(format!("{root}/src/Main.java"))
        .arg("-d")
        .arg(format!("{root}/bin"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    javac_ex.status().unwrap();
}
