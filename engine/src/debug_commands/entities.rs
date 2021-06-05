use anyhow::{anyhow, Context};
use clap::{App, ArgMatches, load_yaml};

use ecs::{Component, Entities, Entity, Resources, Storage};

use crate::{CommandTrait, HeadlessBackend};
use crate::components::{Camera, Info, Model, Status, UiModel, Renderable, RenderableType};
use super::Error;
use serde::{Serialize, Deserialize};
use crate::resources::{SceneGraph, GraphicsBackend, AssetDatabase};

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

    fn run(&self, res: &Resources, args: &[String]) -> anyhow::Result<()> {
        let app_yaml = load_yaml!("entities.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;

        if let Some(add_matches) = matches.subcommand_matches("add") {
            let add_renderable = add_matches.is_present("renderable");
            let index = add_matches.value_of("index")
                .ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entities = res.borrow::<Entities>();
            let entity = entities.try_get(index).ok_or(Error::EntityNotFound(index))?;

            if add_renderable {
                let mut factory = res.borrow_mut::<GraphicsBackend<HeadlessBackend>>();
                let assets = res.borrow::<AssetDatabase>();
                let renderable = Renderable::builder()
                    .with_type(RenderableType::Text)
                    .with_text("Hello, World!")
                    .with_font(assets.find_asset("fonts/SourceSansPro-Regular.ttf")?)
                    .with_vertex_shader(assets.find_asset("shaders/text-vertex.glsl")?)
                    .with_fragment_shader(assets.find_asset("shaders/text-fragment.glsl")?)
                    .build(&mut factory)?;
                res.borrow_components_mut::<Renderable>()
                    .insert(entity, renderable);
            }
        }

        if let Some(create_matches) = matches.subcommand_matches("create") {
            let ui_element = create_matches.is_present("ui");
            let camera = create_matches.is_present("camera");
            let name = create_matches.value_of("name").context("Missing required argument name")?;

            // FIXME: The following can cause issues if adding a UI entity as a child of a world entity or vice versa
            let parent = if let Some(parent_str) = create_matches.value_of("parent") {
                let parent_idx: usize = parent_str.parse().context("The value of argument parent is not a positive integer")?;
                let parent_entity = res.borrow::<Entities>()
                    .try_get(parent_idx)
                    .ok_or(anyhow!("The entity with index {} was not found", parent_idx))?;

                Some(parent_entity)
            } else {
                None
            };

            let new_entity = res.borrow_mut::<Entities>().create();
            res.borrow_components_mut::<Info>().insert(new_entity, Info::new(name, ""));
            res.borrow_components_mut::<Status>().insert(new_entity, Status::default());

            if ui_element {
                res.borrow_components_mut::<UiModel>().insert(new_entity, UiModel::default());
                if let Some(ref p) = parent {
                    res.borrow_mut::<SceneGraph<UiModel>>().insert_child(p, new_entity);
                } else {
                    res.borrow_mut::<SceneGraph<UiModel>>().insert(new_entity);
                }
            } else {
                if camera {
                    res.borrow_components_mut::<Camera>().insert(new_entity, Camera::default());
                }
                res.borrow_components_mut::<Model>().insert(new_entity, Model::default());
                if let Some(ref p) = parent {
                    res.borrow_mut::<SceneGraph<Model>>().insert_child(p, new_entity);
                } else {
                    res.borrow_mut::<SceneGraph<Model>>().insert(new_entity);
                }
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

        if let Some(status_matches) = matches.subcommand_matches("status") {
            let entities = res.borrow::<Entities>();
            let mut statuses = res.borrow_components_mut::<Status>();
            let index = status_matches.value_of("index")
                .ok_or(Error::NoIndexSpecified)?;

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
            }

            if status_matches.is_present("show") {
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

            let (enabled, visible) = statuses.get(entity)
                .map(|s| (s.enabled(), s.visible()))
                .unwrap_or((false, false));
            println!("Enabled: {}, Visible: {}", enabled, visible);
        }

        Ok(())
    }
}
