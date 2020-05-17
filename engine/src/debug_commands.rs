use crate::{
    components::{Camera, Info, Model, Status, UiModel},
    event::EngineEvent,
    resources::SceneGraph,
};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use ecs::{Component, Entities, Entity, EventQueue, Resources, Storage, WorldEvent};
use anyhow::Result;
use thiserror::Error;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("You must specify an entity index if you want to change the status of an entity")]
    NoIndexSpecified,
    #[error("The entity with index {0} was not found")]
    EntityNotFound(usize),
    #[error("The entity with index {0} cannot be enabled")]
    CannotEnableEntity(usize),
    #[error("The entity with index {0} cannot be disabled")]
    CannotDisableEntity(usize),
    #[error("The entity with index {0} cannot be shown")]
    CannotShowEntity(usize),
    #[error("The entity with index {0} cannot be hidden")]
    CannotHideEntity(usize),
}

pub trait CommandTrait: 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, res: &Resources, args: &[String]) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub struct ExitCommand;

impl CommandTrait for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Shuts down the engine (can also be done with Ctrl-C. Tap Ctrl-C twice to force a shutdown)"
    }

    fn run(&self, res: &Resources, _: &[String]) -> Result<()> {
        res.borrow_mut::<EventQueue<EngineEvent>>().send(EngineEvent::Shutdown);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StateCommand;

impl CommandTrait for StateCommand {
    fn name(&self) -> &'static str {
        "state"
    }

    fn description(&self) -> &'static str {
        "Provides access to state serialization functions"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
        let matches = App::new("state")
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                SubCommand::with_name("save")
                    .about("Saves the world state to a file")
                    .setting(AppSettings::DisableVersion)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(
                        Arg::with_name("path")
                            .required(true)
                            .takes_value(true)
                            .validator_os(|s| {
                                Path::new(s)
                                    .parent()
                                    .filter(|p| p.is_dir())
                                    .map(|_| ())
                                    .ok_or(OsString::from("expected a path to a new or existing writable file"))
                            })
                            .help("Sets the path of the file to write to"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("load")
                    .about("Loads the world state from a file")
                    .setting(AppSettings::DisableVersion)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(
                        Arg::with_name("path")
                            .required(true)
                            .takes_value(true)
                            .validator_os(|s| {
                                if Path::new(s).is_file() {
                                    Ok(())
                                } else {
                                    Err(OsString::from("expected a path to an existing readable file"))
                                }
                            })
                            .help("Sets the path of the file to read from"),
                    ),
            )
            .get_matches_from_safe(args)?;

        if let Some(save_matches) = matches.subcommand_matches("save") {
            if let Some(path_str) = save_matches.value_of("path") {
                let path = PathBuf::from(path_str);
                res.borrow_mut::<EventQueue<WorldEvent>>()
                    .send(WorldEvent::Serialize(path));
            }
        }

        if let Some(load_matches) = matches.subcommand_matches("load") {
            if let Some(path_str) = load_matches.value_of("path") {
                let path = PathBuf::from(path_str);
                res.borrow_mut::<EventQueue<WorldEvent>>()
                    .send(WorldEvent::Deserialize(path));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraCommand;

impl CommandTrait for CameraCommand {
    fn name(&self) -> &'static str {
        "camera"
    }

    fn description(&self) -> &'static str {
        "Provides access to the camera"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
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
            let cameras = res.borrow_components::<Camera>();

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

#[derive(Debug, Clone, Copy)]
pub struct EntityCommand;

impl EntityCommand {
    fn list_entity(
        &self,
        args: &ArgMatches,
        cameras: &<Camera as Component>::Storage,
        infos: &<Info as Component>::Storage,
        statuses: &<Status as Component>::Storage,
        world_graph: &SceneGraph<Model>,
        ui_graph: &SceneGraph<UiModel>,
        entity: &Entity,
    ) {
        let mut output = String::new();

        if args.is_present("names") || args.is_present("statuses") || args.is_present("positions") {
            output.push(':');
        }

        if args.is_present("names") {
            if let Some(i) = infos.get(entity) {
                output.push(' ');
                output.push_str(i.name());
            } else {
                output.push_str(" (no name)");
            }
        }

        if args.is_present("statuses") {
            if let Some(s) = statuses.get(entity) {
                output.push_str(&format!(
                    " status=({}, {})",
                    if s.enabled() { "enabled" } else { "disabled" },
                    if s.visible() { "visible" } else { "hidden" }
                ));
            } else {
                output.push_str(" (no status)");
            }
        }

        if args.is_present("positions") {
            if let Some(m) = world_graph.get(entity) {
                let pos = m.position();
                output.push_str(&format!(" world-pos=[{}, {}, {}]", pos.x, pos.y, pos.z));
            } else if let Some(m) = ui_graph.get(entity) {
                let pos = m.position();
                output.push_str(&format!(" ui-pos=[{}, {}]", pos.x, pos.y));
            } else {
                output.push_str(" (no position)");
            }
        }

        if args.is_present("ndc-positions") {
            for (i, camera) in cameras.iter().enumerate() {
                if let Some(m) = world_graph.get(entity) {
                    let pos = camera.world_point_to_ndc(&m.position());
                    output.push_str(&format!(" cam-{}-ndc-pos=[{}, {}, {}]", i, pos.x, pos.y, pos.z));
                } else if let Some(m) = ui_graph.get(entity) {
                    let pos = camera.ui_point_to_ndc(&m.position(), m.depth());
                    output.push_str(&format!(" cam-{}-ndc-pos=[{}, {}, {}]", i, pos.x, pos.y, pos.z));
                } else {
                    output.push_str(" (no ndc position)");
                }
            }
        }

        println!("{}{}", entity, output);
    }
}

impl CommandTrait for EntityCommand {
    fn name(&self) -> &'static str {
        "entity"
    }

    fn description(&self) -> &'static str {
        "Provides access to entities within the world"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
        let matches = App::new("entity")
            .about("Provides access to entities within the world")
            .setting(AppSettings::DisableVersion)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .arg(
                Arg::with_name("index")
                    .short("i")
                    .long("index")
                    .takes_value(true)
                    .help("Specify the index of the desired entity"),
            )
            .subcommand(
                SubCommand::with_name("list")
                    .about("Prints a list of entities")
                    .setting(AppSettings::DisableVersion)
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
                        Arg::with_name("statuses")
                            .short("s")
                            .long("statuses")
                            .help("Displays the statuses of entities"),
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
            .subcommand(
                SubCommand::with_name("status")
                    .about("Updates the status of one or more entities")
                    .setting(AppSettings::DisableVersion)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(
                        Arg::with_name("enable")
                            .short("e")
                            .long("enable")
                            .conflicts_with("disable")
                            .help("Enables the selected entity"),
                    )
                    .arg(
                        Arg::with_name("disable")
                            .short("d")
                            .long("disable")
                            .conflicts_with("enable")
                            .help("Disables the selected entity"),
                    )
                    .arg(
                        Arg::with_name("show")
                            .short("s")
                            .long("show")
                            .conflicts_with("hide")
                            .help("Shows the selected entity"),
                    )
                    .arg(
                        Arg::with_name("hide")
                            .short("h")
                            .long("hide")
                            .conflicts_with("show")
                            .help("Hides the selected entity"),
                    ),
            )
            .get_matches_from_safe(args)?;

        if let Some(list_matches) = matches.subcommand_matches("list") {
            let entities = res.borrow::<Entities>().iter().collect::<Vec<_>>();
            let cameras = res.borrow_components::<Camera>();
            let infos = res.borrow_components::<Info>();
            let statuses = res.borrow_components::<Status>();
            let world_graph = res.borrow::<SceneGraph<Model>>();
            let ui_graph = res.borrow::<SceneGraph<UiModel>>();

            if list_matches.is_present("count") {
                println!("Loaded entities: {}", entities.len());
            }

            if let Some(index) = matches.value_of("index") {
                let index: usize = index.parse()?;
                let entity = entities
                    .get(index)
                    .ok_or(Error::EntityNotFound(index))?;
                self.list_entity(
                    list_matches,
                    &cameras,
                    &infos,
                    &statuses,
                    &world_graph,
                    &ui_graph,
                    entity,
                );
            } else {
                for entity in &entities {
                    self.list_entity(
                        list_matches,
                        &cameras,
                        &infos,
                        &statuses,
                        &world_graph,
                        &ui_graph,
                        entity,
                    );
                }
            }
        }

        if let Some(status_matches) = matches.subcommand_matches("status") {
            let entities = res.borrow::<Entities>().iter().collect::<Vec<_>>();
            let mut statuses = res.borrow_components_mut::<Status>();

            if let Some(index) = matches.value_of("index") {
                let index: usize = index.parse()?;
                let entity = entities
                    .get(index)
                    .ok_or(Error::EntityNotFound(index))?;

                if status_matches.is_present("enable") {
                    statuses
                        .get_mut(entity)
                        .map(|s| s.enable())
                        .ok_or(Error::CannotEnableEntity(index))?;
                } else if status_matches.is_present("disable") {
                    statuses
                        .get_mut(entity)
                        .map(|s| s.disable())
                        .ok_or(Error::CannotDisableEntity(index))?;
                } else if status_matches.is_present("show") {
                    statuses
                        .get_mut(entity)
                        .map(|s| s.show())
                        .ok_or(Error::CannotShowEntity(index))?;
                } else if status_matches.is_present("hide") {
                    statuses
                        .get_mut(entity)
                        .map(|s| s.hide())
                        .ok_or(Error::CannotHideEntity(index))?;
                }
            } else {
                return Err(From::from(Error::NoIndexSpecified));
            }
        }

        Ok(())
    }
}
