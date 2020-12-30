use crate::{
    debug_commands::{CameraCommand, CommandTrait, EntityCommand, ExitCommand, StateCommand},
    event::EngineEvent,
};
use anyhow::Result;
use ecs::{EventQueue, ReceiverId, Resources, System};
use log::trace;
use std::{collections::HashMap, time::Duration};
use thiserror::Error;

pub struct DebugShell {
    commands: HashMap<&'static str, Box<dyn CommandTrait>>,
    receiver: ReceiverId<EngineEvent>,
    terminator: &'static str,
}

impl DebugShell {
    pub fn new(queue: &mut EventQueue<EngineEvent>, terminator: Option<&'static str>) -> Self {
        trace!("DebugShell subscribing to EventQueue<EngineEvent>");
        let mut sys = DebugShell {
            commands: HashMap::new(),
            receiver: queue.subscribe(),
            terminator: terminator.unwrap_or(";"),
        };

        sys.add_command(ExitCommand);
        sys.add_command(CameraCommand);
        sys.add_command(EntityCommand);
        sys.add_command(StateCommand);

        sys
    }

    pub fn add_command<C: CommandTrait>(&mut self, command: C) {
        self.commands.insert(command.name(), Box::new(command));
    }

    fn interpret(&self, res: &Resources, tokens: &[String]) -> Result<()> {
        // Iterate over all commands
        for token_group in tokens.split(|t| t == self.terminator) {
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
