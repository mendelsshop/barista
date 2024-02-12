use std::process::{Command, Stdio};

use javaup::config;

use crate::{config::Config, utils::unless_exists};

impl Config {
    pub fn jar(&self, path: &str) {
        self.fetch();
        let binding = crate::config::get_root_path().unwrap();
        let root = binding.display();
        let mut java_bin = config::jdkdir();

        let java_config = config::config_file();
        java_bin.push(java_config.default_jdk.clone().unwrap().distribution);
        java_bin.push(java_config.default_jdk.unwrap().version);
        java_bin.push("bin");
        let mut jar_bin = java_bin.clone();
        #[cfg(target_os = "windows")]
        {
            jar_bin.push("jar.exe");
            java_bin.push("javac.exe");
        }
        #[cfg(not(target_os = "windows"))]
        {
            java_bin.push("javac");
            jar_bin.push("jar");
        }
        let mut binding = Command::new(java_bin);
        unless_exists(format!("{path}/src/Library.java"), || {
            panic!("not bin target found")
        });
        let javac_ex = binding
            .arg("-cp")
            .arg(format!("{root}/lib/*",))
            .arg("--source-path")
            .arg(format!("{path}/src"))
            .arg(format!("{path}/src/Library.java"))
            .arg("-d")
            .arg(format!("{root}/bin/{}/", self.brew().name()))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        javac_ex.status().unwrap();
        let mut binding = Command::new(jar_bin);
        let bindings = binding.current_dir(format!("bin"));
        bindings
            .arg("-cf")
            .arg(format!(
                "../lib/{}-{}.{}",
                self.brew().name(),
                self.brew().version(),
                "jar"
            ))
            .arg(self.brew().name())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .unwrap();
    }
}
