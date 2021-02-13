use crate::{
    debug_commands::{CameraCommand, CommandTrait, EntityCommand, ExitCommand, StateCommand},
    event::EngineEvent,
};
use anyhow::Result;
use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};
use log::trace;
use std::{collections::HashMap, time::Duration};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::marker::PhantomData;
use crate::resources::SettingsTrait;

#[derive(Serialize, Deserialize)]
pub struct DebugShell<S> {
    #[serde(skip, default = "default_commands")]
    commands: HashMap<&'static str, Box<dyn CommandTrait>>,
    receiver: ReceiverId<EngineEvent>,
    #[serde(skip)]
    _s: PhantomData<S>,
}

impl<S> std::fmt::Debug for DebugShell<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "DebugShell {{ commands: {:?}, receiver: {:?} }}",
            self.commands.keys(),
            self.receiver,
        )
    }
}

impl<S> WithResources for DebugShell<S> {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>()
            .subscribe::<Self>();

        DebugShell {
            commands: default_commands(),
            receiver,
            _s: PhantomData::default(),
        }
    }
}

impl<S> DebugShell<S>
where
    S: SettingsTrait,
{
    pub fn add_command<C: CommandTrait>(&mut self, command: C) {
        self.commands.insert(command.name(), Box::new(command));
    }

    fn interpret(&self, res: &Resources, tokens: &[String]) -> Result<()> {
        let terminator = res.borrow::<S>().command_punctuation();

        // Iterate over all commands
        for token_group in tokens.split(|t| t.len() == 1 && t.contains(terminator)) {
            // Determine the current command name
            let command_name = token_group[0].as_str();

            // Find and execute the appropriate matching command
            if command_name == "help" {
                self.command_help()?;
            } else {
                self.commands
                    .get(command_name)
                    .ok_or(DebugShellError::CommandNotFound(command_name.to_string()).into())
                    .and_then(|c| c.run(res, token_group))?;
            }
        }

        Ok(())
    }

    fn command_help(&self) -> Result<()> {
        let mut output =
            String::from("For more information on a specific command, type COMMAND -h\n");
        for (k, v) in &self.commands {
            output.push_str(k);
            output.push_str(": ");
            output.push_str(v.description());
            output.push('\n');
        }
        print!("{}", output);

        Ok(())
    }
}

impl<S> System for DebugShell<S>
where
    S: SettingsTrait + 'static,
{
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res
            .borrow_mut::<EventQueue<EngineEvent>>()
            .receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::Command(ref tokens) => self
                    .interpret(res, tokens)
                    .unwrap_or_else(|e| eprintln!("{}", e)),
                _ => (),
            }
        }
    }
}

#[derive(Debug, Error)]
enum DebugShellError {
    #[error("'{0}' is not a recognized builtin or command")]
    CommandNotFound(String),
}

fn box_command<C: CommandTrait>(command: C) -> Box<dyn CommandTrait + 'static> {
    Box::new(command)
}

fn default_commands() -> HashMap<&'static str, Box<dyn CommandTrait>> {
    let mut commands = HashMap::with_capacity(4);
    commands.insert(ExitCommand.name(), box_command(ExitCommand));
    commands.insert(CameraCommand.name(), box_command(CameraCommand));
    commands.insert(EntityCommand.name(), box_command(EntityCommand));
    commands.insert(StateCommand.name(), box_command(StateCommand));
    commands
}
