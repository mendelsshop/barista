use clap::{Parser, Subcommand};
use config::{config_file, init, root_dir, unless_exists, write_config};
use serde::{Deserialize, Serialize};
// https://github.com/foojayio/discoapi
const DEFALT_VERSION: &str = "17";
const DEFALT_DISTRIBUTION: &str = "temurin";

mod config;
mod request_builder;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    command: CommandType,
}
#[derive(Subcommand, Clone, Debug)]
pub enum CommandType {
    /// Install JDK
    Install(ToolChain),
    /// List available JDKs or distributions
    List {
        #[clap(subcommand)]
        listtype: ListType,
    },
    /// Set default JDK
    Default(ToolChain),
}
#[derive(Subcommand, Clone, Debug)]
pub enum ListType {
    /// List the available versions from the given distribution
    Versions {
        #[clap(default_value=DEFALT_DISTRIBUTION)]
        distribution: String,
    },
    /// List the available distributions
    Distributions,
}
fn main() {
    unless_exists(&root_dir(), init);
    let args = Args::parse();
    match args.command {
        CommandType::Install(jdkinfo) => request_builder::RequestBuilder::new()
            .install(jdkinfo)
            .execute(),
        CommandType::List { listtype } => request_builder::RequestBuilder::new()
            .list(listtype)
            .execute(),
        CommandType::Default(jdk) => {
            let mut config = config_file();
            config.default_jdk = Some(jdk);
            write_config(&config)
        }
    }
}

#[derive(clap::Parser, Clone, Debug, Deserialize, Serialize)]
pub struct ToolChain {
    #[clap(default_value=DEFALT_VERSION)]
    version: String,
    #[clap(default_value=DEFALT_DISTRIBUTION)]
    distribution: String,
}
