use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

pub mod config;
pub mod request_builder;
const DEFALT_VERSION: &str = "17";
const DEFALT_DISTRIBUTION: &str = "temurin";

#[derive(clap::Parser, Clone, Debug, Deserialize, Serialize)]
pub struct ToolChain {
    #[clap(default_value=DEFALT_VERSION)]
    pub version: String,
    #[clap(default_value=DEFALT_DISTRIBUTION)]
    pub distribution: String,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: CommandType,
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
