use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    process::{Command, Stdio},
};

use javaup::config;

use crate::config::Config;

pub fn make_menu() {
    Config::find_and_open_config().unwrap().fetch();
    let binding = crate::config::get_root_path().unwrap();
    let root = binding.display();
    let java_config = config::config_file();
    let mut java_bin = config::jdkdir();
    // create docs dir if doest exist
    let create_dir = fs::create_dir(format!("{}/doc", root));
    if create_dir
        .as_ref()
        .is_err_and(|e| e.kind() != ErrorKind::AlreadyExists)
    {
        panic!("error creating doc dir: {}", create_dir.unwrap_err())
    }
    java_bin.push(java_config.default_jdk.clone().unwrap().distribution);
    java_bin.push(java_config.default_jdk.unwrap().version);
    java_bin.push("bin");
    #[cfg(target_os = "windows")]
    java_bin.push("javadoc.exe");
    #[cfg(not(target_os = "windows"))]
    java_bin.push("javadoc");

    // TODO: make this not hard coded also allow multiple all parts to be deocmneted
    let bin_path =
        PathBuf::from_iter(([root.to_string(), "src".to_string(), "Main".to_string(),  "Main.java".to_string()]));
    let bin_path = bin_path.display();
    let mut binding = Command::new(java_bin);
    let javac_ex = binding
        .arg("-d")
        .arg(format!("{root}/doc"))
        .arg("--source-path")
        .arg(format!("{root}/src/"))
        .arg(format!("{bin_path}"))
        // ignoring output untill we have good way to filter/present it
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        ;

    javac_ex.status().unwrap();
}
