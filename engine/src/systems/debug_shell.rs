use crate::{
    debug_commands::{CameraCommand, CommandTrait, EntityCommand, ExitCommand},
    event::EngineEventTrait,
};
use ecs::{EventHandlerSystem, Resources};
use failure::Error;
use std::{collections::HashMap, marker::PhantomData};

pub struct DebugShell<Evt> {
    commands: HashMap<&'static str, Box<dyn CommandTrait>>,
    _evt: PhantomData<Evt>,
}

impl<Evt> Default for DebugShell<Evt>
where
    Evt: EngineEventTrait + 'static,
{
    fn default() -> Self {
        let mut sys = DebugShell {
            commands: HashMap::new(),
            _evt: PhantomData::default(),
        };

        sys.add_command(ExitCommand::<Evt>::default());
        sys.add_command(CameraCommand::default());
        sys.add_command(EntityCommand::default());

        sys
    }
}

impl<Evt> DebugShell<Evt>
where
    Evt: EngineEventTrait + 'static,
{
    pub fn add_command<C: CommandTrait>(&mut self, command: C) {
        self.commands.insert(command.name(), Box::new(command));
    }

    fn interpret(&self, res: &mut Resources, args: &[String]) -> Result<(), Error> {
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

impl<Evt> EventHandlerSystem<Evt> for DebugShell<Evt>
where
    Evt: EngineEventTrait + 'static,
{
    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::command()
    }

    fn run(&mut self, res: &mut Resources, event: &Evt) -> bool {
        if let Some(ref args) = event.command_data() {
            self.interpret(res, args).unwrap_or_else(|e| eprintln!("{}", e));
        }

        true
    }
}

#[derive(Debug, Fail)]
enum DebugShellError {
    #[fail(display = "'{}' is not a recognized builtin or command", _0)]
    CommandNotFound(String),
}
