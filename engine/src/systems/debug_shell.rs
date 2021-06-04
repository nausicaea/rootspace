use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use ecs::{EventQueue, ReceiverId, Resources, SerializationName, System, WithResources};

use crate::{debug_commands, debug_commands::CommandTrait, event::EngineEvent, resources::settings::Settings};

#[derive(Serialize, Deserialize)]
pub struct DebugShell {
    #[serde(skip, default = "crate::debug_commands::default_commands")]
    commands: HashMap<&'static str, Box<dyn CommandTrait>>,
    receiver: ReceiverId<EngineEvent>,
}

impl std::fmt::Debug for DebugShell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "DebugShell {{ commands: {:?}, receiver: {:?} }}",
            self.commands.keys(),
            self.receiver,
        )
    }
}

impl WithResources for DebugShell {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        DebugShell {
            commands: debug_commands::default_commands(),
            receiver,
        }
    }
}

impl DebugShell {
    pub fn add_command<C: CommandTrait>(&mut self, command: C) {
        self.commands.insert(command.name(), Box::new(command));
    }

    fn interpret(&self, res: &Resources, tokens: &[String]) -> Result<()> {
        let terminator = res.borrow::<Settings>().command_punctuation;

        // Iterate over all commands
        for token_group in tokens.split(|t| t.len() == 1 && t.contains(terminator)) {
            if token_group.is_empty() {
                continue;
            }

            // Determine the current command name
            let command_name = token_group[0].as_str();

            // Find and execute the appropriate matching command
            if command_name == "help" {
                self.command_help()?;
            } else {
                self.commands
                    .get(command_name)
                    .ok_or_else(|| DebugShellError::CommandNotFound(command_name.to_string()).into())
                    .and_then(|c| c.run(res, token_group))?;
            }
        }

        Ok(())
    }

    fn command_help(&self) -> Result<()> {
        let mut output = String::from("For more information on a specific command, type COMMAND -h\n");
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
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            if let EngineEvent::Command(ref tokens) = event {
                self.interpret(res, tokens).unwrap_or_else(|e| eprintln!("{}", e))
            }
        }
    }
}

impl SerializationName for DebugShell {}

#[derive(Debug, Error)]
enum DebugShellError {
    #[error("'{0}' is not a recognized builtin or command")]
    CommandNotFound(String),
}
