use std::collections::HashMap;

use anyhow::Result;
use ecs::{impl_registry, Resources};
use thiserror::Error;

pub use self::{
    assets::AssetsCommand, components::ComponentsCommand, entities::EntitiesCommand, exit::ExitCommand,
    states::StatesCommand, stats::StatisticsCommand,
};
use crate::graphics::BackendTrait;

pub mod assets;
pub mod components;
pub mod entities;
pub mod exit;
pub mod states;
pub mod stats;

impl_registry!(CommandRegistry, where Head: CommandTrait + Clone + Copy + Default);

#[derive(Debug, Error)]
pub enum Error {
    #[error("You must specify an entity index if you want to change the status of an entity")]
    NoIndexSpecified,
    #[error("The entity with index {0} was not found")]
    EntityNotFound(usize),
    #[error("The entity with index {0} cannot be enabled")]
    CannotEnableEntity(usize),
    #[error("The entity with index {0} cannot be disabled")]
    CannotDisableEntity(usize),
    #[error("The entity with index {0} cannot be shown")]
    CannotShowEntity(usize),
    #[error("The entity with index {0} cannot be hidden")]
    CannotHideEntity(usize),
    #[error("No arguments were given for the subcommand {}", .0)]
    NoSubcommandArguments(&'static str),
}

pub trait CommandTrait: 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, res: &Resources, args: &[String]) -> Result<()>;
}

impl CommandTrait for () {
    fn name(&self) -> &'static str {
        "()"
    }

    fn description(&self) -> &'static str {
        "Empty operation (eg. a NOP)"
    }

    fn run(&self, _: &Resources, _: &[String]) -> Result<()> {
        Ok(())
    }
}

fn box_command<C: CommandTrait>(command: C) -> Box<dyn CommandTrait + 'static> {
    Box::new(command)
}

pub fn default_commands<B: BackendTrait + 'static>() -> HashMap<&'static str, Box<dyn CommandTrait>> {
    let mut commands = HashMap::with_capacity(4);
    commands.insert(ExitCommand.name(), box_command(ExitCommand));
    commands.insert(EntitiesCommand.name(), box_command(EntitiesCommand));
    commands.insert(StatesCommand.name(), box_command(StatesCommand));
    commands.insert(AssetsCommand.name(), box_command(AssetsCommand));
    commands.insert(ComponentsCommand::<B>::default().name(), box_command(ComponentsCommand::<B>::default()));
    commands.insert(StatisticsCommand.name(), box_command(StatisticsCommand));
    commands
}
