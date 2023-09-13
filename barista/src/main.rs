// main.rs is where the argument parsing is done using [clap](https://crates.io/crates/clap)
// TODO: theres a lot of Error type duplication and sometimes Error types can be more specific
// TODO: check if file/dir is already there and in most cases if so do nothing
use std::process::exit;

use crate::{brew::brew, mix::add_dependency};
use clap::{arg, Parser, Subcommand};
use config::BlendConfig;
use craft::create_new_brew;
use menu::make_menu;
use semver::VersionReq;

mod brew;
mod config;
mod craft;
mod fetch;
mod lock;
mod menu;
mod mix;
mod roast;
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
    /// Document the current [Blend]
    Menu,
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
    version: VersionReq,

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
            unreachable!("should never happen [Case: no author, path, or git url provided]")
        }
    }
}

fn main() {
    let args = Args::parse();
    match args.command {
        CommandType::Brew => brew(),
        CommandType::Roast => {}
        CommandType::Craft { name } => {
            if let Err(e) = create_new_brew(&name) {
                println!("Error creating new Brew\n{e}");
                exit(1);
            }
        }
        CommandType::Mix(blend) => {
            if let Err(e) = add_dependency(&blend.name.clone(), blend.clone().into()) {
                println!("Error adding dependency {blend:?}\n{e:?}");
                exit(1);
            }
        }
        CommandType::Menu => make_menu(),
    }
}
