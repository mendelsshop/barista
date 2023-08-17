use clap::{Parser, Subcommand};
// https://github.com/foojayio/discoapi
const DEFALT_VERSION: &'static str = "17";
const DEFALT_DISTRIBUTION: &'static str = "temurin";

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
    let args = Args::parse();
    match args.command {
        CommandType::Install(jdkinfo) => request_builder::RequestBuilder::new()
            .install(jdkinfo)
            .execute(),
        CommandType::List { listtype } => request_builder::RequestBuilder::new()
            .list(listtype)
            .execute(),
    }
}

#[derive(clap::Parser, Clone, Debug)]
pub struct ToolChain {
    #[clap(default_value=DEFALT_VERSION)]
    version: String,
    #[clap(default_value=DEFALT_DISTRIBUTION)]
    distribution: String,
}
