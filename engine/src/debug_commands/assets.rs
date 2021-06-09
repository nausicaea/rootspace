use anyhow::Context;
use clap::{load_yaml, App};
use ecs::Resources;
use serde::{Deserialize, Serialize};

use crate::{resources::AssetDatabase, CommandTrait};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AssetsCommand;

impl CommandTrait for AssetsCommand {
    fn name(&self) -> &'static str {
        "assets"
    }

    fn description(&self) -> &'static str {
        "Provides access to assets"
    }

    fn run(&self, res: &Resources, args: &[String]) -> anyhow::Result<()> {
        let app_yaml = load_yaml!("assets.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;
        let (subcommand, maybe_subcommand_matches) = matches.subcommand();

        if subcommand == "info" {
            let _scm = maybe_subcommand_matches.context("No arguments were provided to the save subcommand")?;

            let asset_database = res.borrow::<AssetDatabase>();
            println!("Asset tree directory: {:?}", asset_database.asset_directory());
            println!("States directory: {:?}", asset_database.state_directory());
        }

        Ok(())
    }
}
