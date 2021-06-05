use anyhow::Context;
use clap::{load_yaml, App};
use serde::{Deserialize, Serialize};

use ecs::{EventQueue, Resources, WorldEvent};

use crate::{resources::AssetDatabase, CommandTrait};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatesCommand;

impl CommandTrait for StatesCommand {
    fn name(&self) -> &'static str {
        "states"
    }

    fn description(&self) -> &'static str {
        "Provides access to state serialization functions"
    }

    fn run(&self, res: &Resources, args: &[String]) -> anyhow::Result<()> {
        let app_yaml = load_yaml!("states.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;
        let (subcommand, maybe_subcommand_matches) = matches.subcommand();

        if subcommand == "save" {
            let scm = maybe_subcommand_matches.context("No arguments were provided to the save subcommand")?;
            let name = scm.value_of("name").context("Missing required argument 'name'")?;

            let path = res.borrow::<AssetDatabase>().create_state_path(name)?;

            res.borrow_mut::<EventQueue<WorldEvent>>()
                .send(WorldEvent::Serialize(path));
        } else if subcommand == "load" {
            let scm = maybe_subcommand_matches.context("No arguments were provided to the load subcommand")?;
            let name = scm.value_of("name").context("Missing required argument 'name'")?;

            let path = res.borrow::<AssetDatabase>().find_state(name)?;

            res.borrow_mut::<EventQueue<WorldEvent>>()
                .send(WorldEvent::Deserialize(path));
        }

        Ok(())
    }
}
