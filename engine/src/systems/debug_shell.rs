use failure::Error;
use components::model::Model;
use context::SceneGraphTrait;
use std::marker::PhantomData;
use ecs::{Entity, DatabaseTrait, EventManagerTrait, SystemTrait, LoopStage};
use event::{Event, EventFlag, EventData};
use debug_commands::{CommandTrait, ExitCommand, EntityCommand};
use std::collections::HashMap;

pub struct DebugShell<Ctx> {
    commands: HashMap<&'static str, Box<dyn CommandTrait<Ctx>>>,
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> Default for DebugShell<Ctx>
where
    Ctx: EventManagerTrait<Event> + DatabaseTrait + SceneGraphTrait<Entity, Model> + 'static,
{
    fn default() -> Self {
        let mut sys = DebugShell {
            commands: HashMap::new(),
            _ctx: PhantomData::default(),
        };

        sys.add_command(ExitCommand::default());
        sys.add_command(EntityCommand::default());

        sys
    }
}

impl<Ctx> DebugShell<Ctx>
where
    Ctx: EventManagerTrait<Event> + DatabaseTrait + SceneGraphTrait<Entity, Model> + 'static,
{
    fn add_command<C: CommandTrait<Ctx>>(&mut self, command: C) {
        self.commands.insert(command.name(), Box::new(command));
    }

    fn interpret(&self, ctx: &mut Ctx, args: &[String]) -> Result<(), Error> {
        if !args.is_empty() {
            let command_name = args[0].as_str();
            if command_name == "help" {
                self.command_help()
            } else {
                self.commands.get(command_name)
                    .ok_or(DebugShellError::CommandNotFound(command_name.to_string()).into())
                    .and_then(|c| c.run(ctx, args))
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

impl<Ctx> SystemTrait<Ctx, Event> for DebugShell<Ctx>
where
    Ctx: EventManagerTrait<Event> + DatabaseTrait + SceneGraphTrait<Entity, Model> + 'static,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> EventFlag {
        EventFlag::COMMAND
    }

    fn handle_event(&mut self, ctx: &mut Ctx, event: &Event) -> Result<bool, Error> {
        if let EventData::Command(ref args) = event.data() {
            self.interpret(ctx, args)
                .unwrap_or_else(|e| eprintln!("{}", e));
        }

        Ok(true)
    }
}

#[derive(Debug, Fail)]
enum DebugShellError {
    #[fail(display = "'{}' is not a recognized builtin or command", _0)]
    CommandNotFound(String),
}
