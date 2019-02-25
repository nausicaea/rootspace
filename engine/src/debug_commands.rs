use clap::{App, AppSettings, Arg, SubCommand};
use crate::components::{camera::Camera, info::Info};
use ecs::{Resources, EventManager, VecStorage, Entities};
use crate::event::EngineEventTrait;
use failure::Error;
use std::marker::PhantomData;

pub trait CommandTrait: 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, res: &mut Resources, args: &[String]) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct ExitCommand<Evt> {
    _evt: PhantomData<Evt>,
}

impl<Evt> Default for ExitCommand<Evt> {
    fn default() -> Self {
        ExitCommand {
            _evt: PhantomData::default(),
        }
    }
}

impl<Evt> CommandTrait for ExitCommand<Evt>
where
    Evt: EngineEventTrait,
{
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Shuts down the engine (can also be done with Ctrl-C. Tap Ctrl-C twice to force a shutdown)"
    }

    fn run(&self, res: &mut Resources, _: &[String]) -> Result<(), Error> {
        let mgr = res.get_mut::<EventManager<Evt>>()
            .expect("Could not find the event manager");
        mgr.dispatch_later(Evt::new_shutdown());
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CameraCommand;

impl CommandTrait for CameraCommand {
    fn name(&self) -> &'static str {
        "camera"
    }

    fn description(&self) -> &'static str {
        "Provides access to the camera"
    }

    fn run(&self, res: &mut Resources, args: &[String]) -> Result<(), Error> {
        let matches = App::new("camera")
            .about("Provides access to the camera")
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                SubCommand::with_name("info")
                    .about("Prints camera settings")
                    .setting(AppSettings::DisableVersion)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(
                        Arg::with_name("position")
                            .short("p")
                            .long("position")
                            .help("Displays the position of the camera"),
                    )
                    .arg(
                        Arg::with_name("dimensions")
                            .short("d")
                            .long("dimensions")
                            .help("Display the viewport dimensions"),
                    ),
            )
            .get_matches_from_safe(args)?;

        if let Some(info_matches) = matches.subcommand_matches("info") {
            let cameras = res.get::<VecStorage<Camera>>()
                .expect("Could not find the cameras");

            for (i, cam) in cameras.iter().enumerate() {
                println!("Camera {}:", i);

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
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntityCommand;

impl CommandTrait for EntityCommand {
    fn name(&self) -> &'static str {
        "entity"
    }

    fn description(&self) -> &'static str {
        "Provides access to entities within the world"
    }

    fn run(&self, res: &mut Resources, args: &[String]) -> Result<(), Error> {
        let matches = App::new("entity")
            .about("Provides access to entities within the world")
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                SubCommand::with_name("list")
                    .about("Prints a list of entities")
                    .setting(AppSettings::DisableVersion)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(
                        Arg::with_name("count")
                            .short("c")
                            .long("count")
                            .help("Displays the number of entities"),
                    )
                    .arg(
                        Arg::with_name("names")
                            .short("n")
                            .long("names")
                            .help("Displays the names of entities"),
                    )
                    .arg(
                        Arg::with_name("positions")
                            .short("p")
                            .long("positions")
                            .help("Displays the absolute positions of entities"),
                    )
                    .arg(
                        Arg::with_name("ndc-positions")
                            .short("q")
                            .long("ndc-positions")
                            .help("Displays the positions of entities in NDC space"),
                    ),
            )
            .get_matches_from_safe(args)?;

        if let Some(list_matches) = matches.subcommand_matches("list") {
            let cameras = res.get::<VecStorage<Camera>>()
                .expect("Could not find the cameras");
            let entities: Vec<_> = res.get::<Entities>()
                .expect("Could not find the entity register")
                .iter()
                .collect();
            let infos = res.get::<VecStorage<Info>>()
                .expect("Could not find the Info component storage");

            if list_matches.is_present("count") {
                println!("Loaded entities: {}", entities.len());
            }

            for camera in cameras.iter() {
                for entity in &entities {
                    let mut output = String::new();

                    if list_matches.is_present("names") || list_matches.is_present("positions") {
                        output.push(':');
                    }

                    if list_matches.is_present("names") {
                        if let Some(i) = infos.get(entity) {
                            output.push(' ');
                            output.push_str(i.name());
                        } else {
                            output.push_str(" (no name)");
                        }
                    }

                    if list_matches.is_present("positions") {
                        unimplemented!();
                        // if let Some(m) = ctx.get_world_node(entity) {
                        //     let pos = m.position();
                        //     output.push_str(&format!(" world-pos=[{}, {}, {}]", pos.x, pos.y, pos.z));
                        // } else if let Some(m) = ctx.get_ui_node(entity) {
                        //     let pos = m.position();
                        //     output.push_str(&format!(" ui-pos=[{}, {}]", pos.x, pos.y));
                        // } else {
                        //     output.push_str(" (no position)");
                        // }
                    }

                    if list_matches.is_present("ndc-positions") {
                        unimplemented!();
                        // if let Some(m) = ctx.get_world_node(entity) {
                        //     let pos = camera.world_point_to_ndc(&m.position());
                        //     output.push_str(&format!(" ndc-pos=[{}, {}, {}]", pos.x, pos.y, pos.z));
                        // } else if let Some(m) = ctx.get_ui_node(entity) {
                        //     let pos = camera.ui_point_to_ndc(&m.position(), m.depth());
                        //     output.push_str(&format!(" ndc-pos=[{}, {}, {}]", pos.x, pos.y, pos.z));
                        // } else {
                        //     output.push_str(" (no ndc position)");
                        // }
                    }

                    println!("{}{}", entity, output);
                }
            }
        }

        Ok(())
    }
}
