use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use ecs::Resources;
use engine::{AssetMutTrait, CommandTrait};

use crate::assets::FileSystem;

#[derive(Debug, Parser)]
#[clap(name = "fs")]
#[clap(about = "Accesses file system assets", long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Creates a new file system asset")]
    New {
        #[clap(help = "Sets the path of the file to write to")]
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct FileSystemCommand;

impl CommandTrait for FileSystemCommand {
    fn name(&self) -> &'static str {
        "fs"
    }

    fn description(&self) -> &'static str {
        "Accesses file system assets"
    }

    fn run(&self, _: &Resources, args: &[String]) -> Result<()> {
        let matches = Args::try_parse_from(args)?;

        match &matches.subcommand {
            Subcommands::New { path } => {
                FileSystem::default().to_path(path)?;
            },
        }

        Ok(())
    }
}
