use context::SceneGraphTrait;
use debug_commands::{CameraCommand, CommandTrait, EntityCommand, ExitCommand};
use ecs::{DatabaseTrait, EventManagerTrait, LoopStage, SystemTrait};
use event::EngineEventTrait;
use failure::Error;
use std::{collections::HashMap, marker::PhantomData};

pub struct DebugShell<Ctx, Evt> {
    commands: HashMap<&'static str, Box<dyn CommandTrait<Ctx>>>,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt> Default for DebugShell<Ctx, Evt>
where
    Ctx: EventManagerTrait<Evt> + DatabaseTrait + SceneGraphTrait + 'static,
    Evt: EngineEventTrait + 'static,
{
    fn default() -> Self {
        let mut sys = DebugShell {
            commands: HashMap::new(),
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        };

        sys.add_command(ExitCommand::default());
        sys.add_command(CameraCommand::default());
        sys.add_command(EntityCommand::default());

        sys
    }
}

impl<Ctx, Evt> DebugShell<Ctx, Evt>
where
    Ctx: EventManagerTrait<Evt> + DatabaseTrait + SceneGraphTrait + 'static,
    Evt: EngineEventTrait + 'static,
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
                self.commands
                    .get(command_name)
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

impl<Ctx, Evt> SystemTrait<Ctx, Evt> for DebugShell<Ctx, Evt>
where
    Ctx: EventManagerTrait<Evt> + DatabaseTrait + SceneGraphTrait + 'static,
    Evt: EngineEventTrait + 'static,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::command()
    }

    fn handle_event(&mut self, ctx: &mut Ctx, event: &Evt) -> Result<bool, Error> {
        if let Some(ref args) = event.command_data() {
            self.interpret(ctx, args).unwrap_or_else(|e| eprintln!("{}", e));
        }

        Ok(true)
    }
}

#[derive(Debug, Fail)]
enum DebugShellError {
    #[fail(display = "'{}' is not a recognized builtin or command", _0)]
    CommandNotFound(String),
}
