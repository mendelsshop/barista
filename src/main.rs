// main.rs is where the argument parsing is done using [clap](https://crates.io/crates/clap)

use clap::{Parser, Subcommand};
use config::BlendConfig;
use craft::create_new_brew;

use crate::mix::add_dependency;
mod config;
mod craft;
mod mix;

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
    #[clap(default_value = "*")]
    version: String,
    #[clap(flatten)]
    author: Maven,
    #[clap(subcommand)]
    external_blend: Option<BlendType>,
}

#[derive(clap::Parser, Clone, Debug)]
/// Dependencies from Maven Central
pub struct Maven {
    author: String,
}

#[derive(clap::Subcommand, Clone, Debug)]
pub enum BlendType {
    // Maven needs to be a tuple so it can be used as default for mix see (https://github.com/clap-rs/clap/issues/975)
    Maven(Maven),
    /// Dependencies from a file located or accesable from your computer
    Path {
        path: String,
    },
    /// Dependencies from a git repository
    Git {
        url: String,
    },
}

impl From<Blend> for BlendConfig {
    fn from(value: Blend) -> Self {
        let dep = value
            .external_blend
            .unwrap_or(BlendType::Maven(value.author));
        match dep {
            BlendType::Maven(m) => Self::new_maven(value.version, m.author),
            BlendType::Path { path} => Self::new_path(value.version, path),
            BlendType::Git { url } => Self::new_git(value.version, url),
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        CommandType::Brew => {}
        CommandType::Roast => {}
        CommandType::Craft { name } => create_new_brew(&name),
        CommandType::Mix(blend) => add_dependency(&blend.name.clone(), blend.into()),
    }
}
