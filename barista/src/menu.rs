// javac -cp lib/* src/Main.java
use std::{
    fs,
    process::{Command, Stdio}, io::{ErrorKind, self},
};

use anyhow::Context;
// // javac -c lib/* main & java -c lib/* main
use javaup::config;

use crate::config::Config;

pub fn make_menu() {
    Config::find_and_open_config().unwrap().fetch();
    // TODO: need better way obtaining this brews root directory
    let binding = Config::find_config().unwrap();
    let root = binding.parent().unwrap().display();
    let java_config = config::config_file();
    let mut java_bin = config::jdkdir();
    // create docs dir if doest exist
    let create_dir = fs::create_dir(format!("{}/doc", root));
    if create_dir.as_ref().is_err_and(|e| e.kind() != ErrorKind::AlreadyExists) {
        panic!("error creating doc dir: {}", create_dir.unwrap_err())
    }
    java_bin.push(java_config.default_jdk.clone().unwrap().distribution);
    java_bin.push(java_config.default_jdk.unwrap().version);
    java_bin.push("bin");
    #[cfg(target_os = "windows")]
    java_bin.push("javadoc.exe");
    #[cfg(not(target_os = "windows"))]
    java_bin.push("javadoc");

    let mut binding = Command::new(java_bin);
    let javac_ex = binding
        .arg("-cp")
        .arg(format!("{root}/lib/*",))
        .arg(format!("{root}/src/Main.java"))
        .arg("-d")
        .arg(format!("{root}/doc"))
        // ignoring output untill we have good way to filter/present it
        .stdout(Stdio::null()).stderr(Stdio::null());

    javac_ex.status().unwrap();
}