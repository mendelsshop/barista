use std::process::{Command, Stdio};

use javaup::config;

use crate::{config::Config, roast::roast};

pub fn brew() {
    roast();
    let java_config = config::config_file();
    let mut java_bin = config::jdkdir();

    java_bin.push(java_config.default_jdk.clone().unwrap().distribution);
    java_bin.push(java_config.default_jdk.unwrap().version);
    java_bin.push("bin");
    #[cfg(target_os = "windows")]
    java_bin.push("java.exe");
    #[cfg(not(target_os = "windows"))]
    java_bin.push("java");
    // TODO: need better way obtaining this brews root directory
    let binding = Config::find_config().unwrap();
    let root = binding.parent().unwrap().display();
    let mut binding = Command::new(java_bin);
    let binding = binding
        .arg("-cp")
        .arg(format!("{root}/lib/*.:{root}/bin"))
        .arg("Main")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    binding.output().unwrap();
}
