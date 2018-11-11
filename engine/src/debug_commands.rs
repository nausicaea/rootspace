use failure::Error;
use clap::{App, SubCommand, Arg, AppSettings};
use components::{TransformTrait, info::Info, model::Model};
use ecs::{Entity, DatabaseTrait};
use std::marker::PhantomData;
use ecs::EventManagerTrait;
use event::Event;
use context::SceneGraphTrait;

pub trait CommandTrait<Ctx>: 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, ctx: &mut Ctx, args: &[String]) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct ExitCommand<Ctx> {
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> Default for ExitCommand<Ctx> {
    fn default() -> Self {
        ExitCommand {
            _ctx: PhantomData::default(),
        }
    }
}

impl<Ctx> CommandTrait<Ctx> for ExitCommand<Ctx>
where
    Ctx: EventManagerTrait<Event> + 'static,
{
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Shuts down the engine (can also be done with Ctrl-C. Tap Ctrl-C twice to force a shutdown)"
    }

    fn run(&self, ctx: &mut Ctx, _: &[String]) -> Result<(), Error> {
        ctx.dispatch_later(Event::shutdown());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EntityCommand<Ctx> {
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> Default for EntityCommand<Ctx> {
    fn default() -> Self {
        EntityCommand {
            _ctx: PhantomData::default(),
        }
    }
}

impl<Ctx> CommandTrait<Ctx> for EntityCommand<Ctx>
where
    Ctx: EventManagerTrait<Event> + DatabaseTrait + SceneGraphTrait<Entity, Model> + 'static,
{
    fn name(&self) -> &'static str {
        "entity"
    }

    fn description(&self) -> &'static str {
        "Provides access to entities within the world"
    }

    fn run(&self, ctx: &mut Ctx, args: &[String]) -> Result<(), Error> {
        let matches = App::new("entity")
            .about("Provides access to entities within the world")
            .setting(AppSettings::DisableVersion)
            .subcommand(SubCommand::with_name("list")
                        .about("Prints a list of entities")
                        .setting(AppSettings::DisableVersion)
                        .arg(Arg::with_name("count")
                             .short("c")
                             .long("count")
                             .help("Displays the number of entities"))
                        .arg(Arg::with_name("names")
                             .short("n")
                             .long("names")
                             .help("Displays the names of entities"))
                        .arg(Arg::with_name("positions")
                             .short("p")
                             .long("positions")
                             .help("Displays the positions of entities")))
            .get_matches_from_safe(args)?;

        if let Some(list_matches) = matches.subcommand_matches("list") {
            if list_matches.is_present("count") {
                println!("Loaded entities: {}", ctx.num_entities());
            }
            for entity in ctx.entities() {
                let mut output = String::new();

                if list_matches.is_present("names") || list_matches.is_present("positions") {
                    output.push(':');
                }

                if list_matches.is_present("names") {
                    if let Ok(i) = ctx.get::<Info>(entity) {
                        output.push(' ');
                        output.push_str(i.name());
                    } else {
                        output.push_str(" (no name)");
                    }
                }

                if list_matches.is_present("positions") {
                    if let Some(m) = ctx.get_node(entity) {
                        let pos = m.position();
                        output.push_str(&format!(" [{}, {}, {}] ({})", pos.x, pos.y, pos.z, m.layer()));
                    } else {
                        output.push_str(" (no position)");
                    }
                }

                println!("{}{}", entity, output);
            }
        }

        Ok(())
    }
}
