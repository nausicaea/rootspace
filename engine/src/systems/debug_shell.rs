use crate::{
    debug_commands::{CameraCommand, CommandTrait, EntityCommand, ExitCommand, StateCommand},
    event::EngineEvent,
};
use ecs::{EventQueue, ReceiverId, Resources, System};
use failure::{Error, Fail};
use std::{collections::HashMap, time::Duration};
use log::trace;

pub struct DebugShell {
    commands: HashMap<&'static str, Box<dyn CommandTrait>>,
    receiver: ReceiverId<EngineEvent>,
}

impl DebugShell {
    pub fn new(queue: &mut EventQueue<EngineEvent>) -> Self {
        trace!("DebugShell subscribing to EventQueue<EngineEvent>");
        let mut sys = DebugShell {
            commands: HashMap::new(),
            receiver: queue.subscribe(),
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

    fn interpret(&self, res: &Resources, args: &[String]) -> Result<(), Error> {
        if !args.is_empty() {
            let command_name = args[0].as_str();
            if command_name == "help" {
                self.command_help()
            } else {
                self.commands
                    .get(command_name)
                    .ok_or(DebugShellError::CommandNotFound(command_name.to_string()).into())
                    .and_then(|c| c.run(res, args))
            }
        } else {
            Ok(())
        }
    }

    fn command_help(&self) -> Result<(), Error> {
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
    fn name(&self) -> &'static str {
        "DebugShell"
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::Command(ref args) => self.interpret(res, args).unwrap_or_else(|e| eprintln!("{}", e)),
                _ => (),
            }
        }
    }
}

#[derive(Debug, Fail)]
enum DebugShellError {
    #[fail(display = "'{}' is not a recognized builtin or command", _0)]
    CommandNotFound(String),
}
