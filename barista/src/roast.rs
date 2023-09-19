// javac -cp lib/* src/Main.java
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

// // javac -c lib/* main & java -c lib/* main
use javaup::config;

use crate::{config::Config, utils::unless_exists};

pub fn roast() {
    Config::find_and_open_config().unwrap().fetch();
    let binding = crate::config::get_root_path().unwrap();
    let root = binding.display();
    let java_config = config::config_file();
    let mut java_bin = config::jdkdir();
    unless_exists(Path::new(&format!("{root}/lib")), || {
        fs::create_dir_all(format!("{root}/lib"))
            .expect("Failed to create Brew Library directory (lib) when building")
    });
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
