use failure::Error;
use std::marker::PhantomData;
use ecs::{EventManagerTrait, SystemTrait, LoopStage};
use event::{Event, EventFlag, EventData};

pub struct DebugShell<Ctx> {
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> Default for DebugShell<Ctx> {
    fn default() -> Self {
        DebugShell {
            _ctx: PhantomData::default(),
        }
    }
}

impl<Ctx> DebugShell<Ctx>
where
    Ctx: EventManagerTrait<Event>,
{
    fn interpret<S: AsRef<str>>(&self, ctx: &mut Ctx, args: &[S]) -> Result<(), DebugShellError> {
        if !args.is_empty() {
            match args[0].as_ref() {
                "help" => self.command_help(),
                "exit" => self.command_exit(ctx),
                _ => Err(DebugShellError::CommandNotFound(args[0].as_ref().to_string())),
            }
        } else {
            Ok(())
        }
    }

    fn command_help(&self) -> Result<(), DebugShellError> {
        print!(
        "For more information on a specific command, type COMMAND-NAME --help.\
        \nhelp: Prints this message\
        \nexit: Shuts down the engine (can also be done with Ctrl-C. Tap Ctrl-C twice to force a shutdown)\
        \n");

        Ok(())
    }

    fn command_exit(&self, ctx: &mut Ctx) -> Result<(), DebugShellError> {
        ctx.dispatch_later(Event::shutdown());
        Ok(())
    }
}

impl<Ctx> SystemTrait<Ctx, Event> for DebugShell<Ctx>
where
    Ctx: EventManagerTrait<Event>,
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
    #[fail(display = "The required argument '{}' is missing for command '{}'", _1, _0)]
    MissingArgument(String, String),
}
