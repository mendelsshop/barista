// main.rs is where the argument parsing is done using [clap](https://crates.io/crates/clap)

use clap::{arg, Parser, Subcommand};
use config::{BlendConfig, Config};
use craft::create_new_brew;

use crate::mix::add_dependency;
mod config;
mod craft;
mod fetch;
mod lock;
mod mix;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: CommandType,
}
#[derive(Subcommand, Clone, Debug)]
pub enum CommandType {
    /// Build and run the current [Brew] (package)
    Brew,
    /// Build the current [Brew] (package)
    Roast,
    /// Create a new [Brew] (package) with the given name
    Craft { name: String },
    /// Add a new [Blend] (dependency) to the current brew
    Mix(Blend),
}
#[derive(clap::Parser, Clone, Debug)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Blend {
    /// Blend (dependency) name
    name: String,
    /// Blend (dependency) author (required if path or git not specified)
    #[clap(required_unless_present_any(["git", "path"]),conflicts_with_all(["git", "path"]))]
    author: Option<String>,
    #[clap(default_value = "*")]
    version: String,

    #[arg(
        long,
        value_name = "url",
        conflicts_with_all(["path", "author"])
    )]
    git: Option<String>,
    #[arg(long, conflicts_with_all(["git", "author"]))]
    path: Option<String>,
}

#[derive(clap::Parser, Clone, Debug)]
/// Dependencies from Maven Central
pub struct Maven {
    author: String,
}

impl From<Blend> for BlendConfig {
    fn from(value: Blend) -> Self {
        if let Some(path) = value.author {
            Self::new_maven(value.version, path)
        } else if let Some(path) = value.git {
            Self::new_git(value.version, path)
        } else if let Some(path) = value.path {
            Self::new_path(value.version, path)
        } else {
            panic!("should never happen [Case: no author, path, or git url provided]")
        }
    }
}

fn main() {
    let args = Args::parse();
    match args.command {
        CommandType::Brew => {}
        CommandType::Roast => Config::find_and_open_config().unwrap().fetch(),
        CommandType::Craft { name } => create_new_brew(&name),
        CommandType::Mix(blend) => add_dependency(&blend.name.clone(), blend.into()),
    }
}
