use clap::{App, AppSettings, Arg, SubCommand};
use components::{camera::Camera, info::Info, model::Model, TransformTrait};
use context::SceneGraphTrait;
use ecs::EventManagerTrait;
use ecs::{DatabaseTrait, Entity};
use event::EngineEventTrait;
use failure::Error;
use std::marker::PhantomData;

pub trait CommandTrait<Ctx>: 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, ctx: &mut Ctx, args: &[String]) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct ExitCommand<Ctx, Evt> {
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt> Default for ExitCommand<Ctx, Evt> {
    fn default() -> Self {
        ExitCommand {
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        }
    }
}

impl<Ctx, Evt> CommandTrait<Ctx> for ExitCommand<Ctx, Evt>
where
    Ctx: EventManagerTrait<Evt> + 'static,
    Evt: EngineEventTrait + 'static,
{
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Shuts down the engine (can also be done with Ctrl-C. Tap Ctrl-C twice to force a shutdown)"
    }

    fn run(&self, ctx: &mut Ctx, _: &[String]) -> Result<(), Error> {
        ctx.dispatch_later(Evt::new_shutdown());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CameraCommand<Ctx> {
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> Default for CameraCommand<Ctx> {
    fn default() -> Self {
        CameraCommand {
            _ctx: PhantomData::default(),
        }
    }
}

impl<Ctx> CommandTrait<Ctx> for CameraCommand<Ctx>
where
    Ctx: DatabaseTrait + 'static,
{
    fn name(&self) -> &'static str {
        "camera"
    }

    fn description(&self) -> &'static str {
        "Provides access to the camera"
    }

    fn run(&self, ctx: &mut Ctx, args: &[String]) -> Result<(), Error> {
        let matches = App::new("camera")
            .about("Provides access to the camera")
            .setting(AppSettings::DisableVersion)
            .subcommand(
                SubCommand::with_name("info")
                    .about("Prints camera settings")
                    .setting(AppSettings::DisableVersion)
                    .arg(
                        Arg::with_name("position")
                            .short("p")
                            .long("position")
                            .help("Displays the position of the camera"),
                    ).arg(
                        Arg::with_name("dimensions")
                            .short("d")
                            .long("dimensions")
                            .help("Display the viewport dimensions"),
                    ),
            ).get_matches_from_safe(args)?;

        if let Some(info_matches) = matches.subcommand_matches("info") {
            let cam = ctx.find::<Camera>()?;

            if info_matches.is_present("position") {
                let pos = cam.position();
                println!("Position: [{}, {}, {}]", pos.x, pos.y, pos.z);
            }

            if info_matches.is_present("dimensions") {
                let dims = cam.dimensions();
                let pdims = cam.physical_dimensions();
                let dpi = cam.dpi_factor();
                println!(
                    "Dimensions: {}x{} (physical={}x{}, DPI-factor={})",
                    dims.0, dims.1, pdims.0, pdims.1, dpi
                );
            }
        }
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
    Ctx: DatabaseTrait + SceneGraphTrait<Entity, Model> + 'static,
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
            .subcommand(
                SubCommand::with_name("list")
                    .about("Prints a list of entities")
                    .setting(AppSettings::DisableVersion)
                    .arg(
                        Arg::with_name("count")
                            .short("c")
                            .long("count")
                            .help("Displays the number of entities"),
                    ).arg(
                        Arg::with_name("names")
                            .short("n")
                            .long("names")
                            .help("Displays the names of entities"),
                    ).arg(
                        Arg::with_name("positions")
                            .short("p")
                            .long("positions")
                            .help("Displays the positions of entities"),
                    ),
            ).get_matches_from_safe(args)?;

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
