use crate::{
    debug_commands::{CameraCommand, CommandTrait, EntityCommand, ExitCommand, StateCommand},
    event::EngineEvent,
};
use anyhow::Result;
use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};
use log::trace;
use std::{collections::HashMap, time::Duration};
use thiserror::Error;
use crate::resources::BackendSettings;

pub struct DebugShell {
    commands: HashMap<&'static str, Box<dyn CommandTrait>>,
    receiver: ReceiverId<EngineEvent>,
    terminator: char,
}

impl std::fmt::Debug for DebugShell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DebugShell {{ commands: {:?}, receiver: {:?}, terminator: {:?}}}", self.commands.keys(), self.receiver, self.terminator)
    }
}

impl WithResources for DebugShell {
    fn with_resources(res: &Resources) -> Self {
        let terminator = res.borrow::<BackendSettings>().command_punctuation;
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>()
            .subscribe::<Self>();

        let mut sys = DebugShell {
            commands: HashMap::new(),
            receiver,
            terminator,
        };

        sys.add_command(ExitCommand);
        sys.add_command(CameraCommand);
        sys.add_command(EntityCommand);
        sys.add_command(StateCommand);

        sys
    }
}

impl DebugShell {
    pub fn add_command<C: CommandTrait>(&mut self, command: C) {
        self.commands.insert(command.name(), Box::new(command));
    }

    fn interpret(&self, res: &Resources, tokens: &[String]) -> Result<()> {
        // Iterate over all commands
        for token_group in tokens.split(|t| t.len() == 1 && t.contains(self.terminator)) {
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

impl System for DebugShell {
    fn name(&self) -> &'static str {
        stringify!(DebugShell)
    }

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
