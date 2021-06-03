use std::path::PathBuf;

use anyhow::{Result, Context};
use clap::{App, ArgMatches, load_yaml};
use thiserror::Error;

use ecs::{world::event::WorldEvent, Component, Entities, Entity, EventQueue, Resources, Storage};

use crate::{
    components::{Camera, Info, Model, Status, UiModel},
    event::EngineEvent,
    resources::SceneGraph,
};
use ecs::impl_registry;
use serde::{Deserialize, Serialize};
use crate::resources::AssetDatabase;

impl_registry!(CommandRegistry, where Head: CommandTrait + Clone + Copy + Default);

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

impl CommandTrait for () {
    fn name(&self) -> &'static str {
        "()"
    }

    fn description(&self) -> &'static str {
        "Empty operation (eg. a NOP)"
    }

    fn run(&self, _: &Resources, _: &[String]) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StatesCommand;

impl CommandTrait for StatesCommand {
    fn name(&self) -> &'static str {
        "states"
    }

    fn description(&self) -> &'static str {
        "Provides access to state serialization functions"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
        let app_yaml = load_yaml!("states.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;
        let (subcommand, maybe_subcommand_matches) = matches.subcommand();

        if subcommand == "save" {
            let scm = maybe_subcommand_matches.context("No arguments were provided to the save subcommand")?;
            let path = scm.value_of("path")
                .map(|p| PathBuf::from(p))
                .context("Missing required argument 'path'")?;

            res.borrow_mut::<EventQueue<WorldEvent>>()
                .send(WorldEvent::Serialize(path));
        } else if subcommand == "load" {
            let scm = maybe_subcommand_matches.context("No arguments were provided to the load subcommand")?;
            let path = scm.value_of("path")
                .map(|p| PathBuf::from(p))
                .context("Missing required argument 'path'")?;

            res.borrow_mut::<EventQueue<WorldEvent>>()
                .send(WorldEvent::Deserialize(path));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AssetsCommand;

impl CommandTrait for AssetsCommand {
    fn name(&self) -> &'static str {
        "assets"
    }

    fn description(&self) -> &'static str {
        "Provides access to assets"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
        let app_yaml = load_yaml!("assets.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;
        let (subcommand, maybe_subcommand_matches) = matches.subcommand();

        if subcommand == "info" {
            let _scm = maybe_subcommand_matches.context("No arguments were provided to the save subcommand")?;

            let asset_database = res.borrow::<AssetDatabase>();
            println!("Asset tree location: {:?}", asset_database.asset_tree());
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CamerasCommand;

impl CamerasCommand {
    fn list_camera(&self, args: &ArgMatches, cam: &Camera, entity: &Entity) {
        let mut output = String::new();

        if args.is_present("dimensions") {
            output.push(':');
        }

        if args.is_present("dimensions") {
            let dims = cam.dimensions();
            let pdims = cam.physical_dimensions();
            let dpi = cam.dpi_factor();
            output.push_str(&format!(
                " dimensions={}x{} physical={}x{} DPI-factor={}",
                dims.0, dims.1, pdims.0, pdims.1, dpi
            ));
        } else {
            output.push_str(" (no dimensions)");
        }

        println!("{}{}", entity.idx(), output);
    }
}

impl CommandTrait for CamerasCommand {
    fn name(&self) -> &'static str {
        "cameras"
    }

    fn description(&self) -> &'static str {
        "Provides access to the camera"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
        let app_yaml = load_yaml!("cameras.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;

        if let Some(info_matches) = matches.subcommand_matches("info") {
            let entities = res.borrow::<Entities>();
            let cameras = res.borrow_components::<Camera>();

            if info_matches.is_present("count") {
                println!("Loaded cameras: {}", cameras.len());
            }

            if let Some(index) = matches.value_of("index") {
                let index: usize = index.parse()?;

                let entity = entities.try_get(index).ok_or(Error::EntityNotFound(index))?;
                let cam = cameras.get(index).ok_or(Error::EntityNotFound(index))?;

                self.list_camera(info_matches, &cam, &entity);
            } else {
                for (idx, cam) in cameras.iter_enum() {
                    self.list_camera(info_matches, &cam, &entities.get(idx));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntitiesCommand;

impl EntitiesCommand {
    fn list_entity(
        &self,
        args: &ArgMatches,
        entities: &Entities,
        cameras: &<Camera as Component>::Storage,
        infos: &<Info as Component>::Storage,
        statuses: &<Status as Component>::Storage,
        models: &<Model as Component>::Storage,
        ui_models: &<UiModel as Component>::Storage,
        world_graph: &SceneGraph<Model>,
        ui_graph: &SceneGraph<UiModel>,
        entity: &Entity,
    ) {
        let mut output = String::new();

        if args.is_present("names")
            || args.is_present("statuses")
            || args.is_present("positions")
            || args.is_present("ndc-positions")
        {
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
            if world_graph.contains(entity) {
                let loc = models
                    .get(entity)
                    .unwrap_or_else(|| panic!("Cannot find the entity {} in the Model components", entity))
                    .position();
                let pos = world_graph.get(entity).position();
                output.push_str(&format!(
                    " local-pos=[{}, {}, {}] world-pos=[{}, {}, {}]",
                    loc.x, loc.y, loc.z, pos.x, pos.y, pos.z
                ));
            } else if ui_graph.contains(entity) {
                let loc = ui_models
                    .get(entity)
                    .unwrap_or_else(|| panic!("Cannot find the entity {} in the UiModel components", entity))
                    .position();
                let pos = ui_graph.get(entity).position();
                output.push_str(&format!(
                    " local-pos=[{}, {}] ui-pos=[{}, {}]",
                    loc.x, loc.y, pos.x, pos.y
                ));
            } else {
                output.push_str(" (no position)");
            }
        }

        if args.is_present("ndc-positions") {
            for (cam_idx, camera) in cameras.iter_enum() {
                let cam_entity = entities.get(cam_idx);
                let cam_model = world_graph.get(&cam_entity);

                if entity == &cam_entity {
                    continue;
                }

                if world_graph.contains(entity) {
                    let m = world_graph.get(entity);
                    let pos = camera.world_point_to_ndc(cam_model, &m.position());
                    output.push_str(&format!(" cam-{}-ndc-pos=[{}, {}, {}]", cam_idx, pos.x, pos.y, pos.z));
                } else if ui_graph.contains(entity) {
                    let m = ui_graph.get(entity);
                    let pos = camera.ui_point_to_ndc(&m.position(), m.depth());
                    output.push_str(&format!(" cam-{}-ndc-pos=[{}, {}, {}]", cam_idx, pos.x, pos.y, pos.z));
                } else {
                    output.push_str(" (no ndc position)");
                }
            }
        }

        println!("{}{}", entity.idx(), output);
    }
}

impl CommandTrait for EntitiesCommand {
    fn name(&self) -> &'static str {
        "entities"
    }

    fn description(&self) -> &'static str {
        "Provides access to entities within the world"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
        let app_yaml = load_yaml!("entities.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;

        if let Some(create_matches) = matches.subcommand_matches("create") {
            let ui_element = create_matches.is_present("ui");

            let new_entity = res.borrow_mut::<Entities>().create();
            res.borrow_components_mut::<Info>().insert(new_entity, Info::default());
            res.borrow_components_mut::<Status>().insert(new_entity, Status::default());

            if ui_element {
                res.borrow_components_mut::<UiModel>().insert(new_entity, UiModel::default());
                res.borrow_mut::<SceneGraph<UiModel>>().insert(new_entity);
            } else {
                res.borrow_components_mut::<Model>().insert(new_entity, Model::default());
                res.borrow_mut::<SceneGraph<Model>>().insert(new_entity);
            }
        }

        if let Some(list_matches) = matches.subcommand_matches("list") {
            let entities = res.borrow::<Entities>();
            let cameras = res.borrow_components::<Camera>();
            let infos = res.borrow_components::<Info>();
            let statuses = res.borrow_components::<Status>();
            let models = res.borrow_components::<Model>();
            let ui_models = res.borrow_components::<UiModel>();
            let world_graph = res.borrow::<SceneGraph<Model>>();
            let ui_graph = res.borrow::<SceneGraph<UiModel>>();

            if list_matches.is_present("count") {
                println!("Loaded entities: {}", entities.len());
            }

            if let Some(index) = matches.value_of("index") {
                let index: usize = index.parse()?;

                let entity = entities.try_get(index).ok_or(Error::EntityNotFound(index))?;

                self.list_entity(
                    list_matches,
                    &entities,
                    &cameras,
                    &infos,
                    &statuses,
                    &models,
                    &ui_models,
                    &world_graph,
                    &ui_graph,
                    &entity,
                );
            } else {
                for entity in &*entities {
                    self.list_entity(
                        list_matches,
                        &entities,
                        &cameras,
                        &infos,
                        &statuses,
                        &models,
                        &ui_models,
                        &world_graph,
                        &ui_graph,
                        &entity,
                    );
                }
            }
        }

        if let Some(status_matches) = matches.subcommand_matches("status") {
            let entities = res.borrow::<Entities>();
            let mut statuses = res.borrow_components_mut::<Status>();

            if let Some(index) = matches.value_of("index") {
                let index: usize = index.parse()?;
                let entity = entities.try_get(index).ok_or(Error::EntityNotFound(index))?;

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
