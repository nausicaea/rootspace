use anyhow::{anyhow, Context};
use clap::{load_yaml, App};

use ecs::{Entities, Resources, Storage, EventQueue};

use super::Error;
use crate::{components::{Camera, Info, Model, Renderable, RenderableType, Status, UiModel}, resources::{AssetDatabase, GraphicsBackend, SceneGraph}, CommandTrait, HeadlessBackend, EngineEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntitiesCommand;

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

        if let Some(info_matches) = matches.subcommand_matches("info") {
            let index = info_matches.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entities = res.borrow::<Entities>();
            let entity = entities.try_get(index).ok_or(Error::EntityNotFound(index))?;

            if let Some(ic) = res.borrow_components::<Info>().get(&entity) {
                println!("Name: {}, Description: {}", ic.name(), ic.description());
            }

            if let Some(sc) = res.borrow_components::<Status>().get(&entity) {
                println!("Enabled: {}, Visible: {}", sc.enabled(), sc.visible());
            }

            if let Some(mc) = res.borrow_components::<Model>().get(&entity) {
                println!("LOCAL - Position: {:?}, Orientation: {}, Scale: {:?}", mc.position().coords, mc.orientation(), mc.scale());
            }

            if let Some(sgmc) = res.borrow::<SceneGraph<Model>>().get(&entity) {
                println!("GLOBAL - Position: {:?}, Orientation: {}, Scale: {:?}", sgmc.position().coords, sgmc.orientation(), sgmc.scale());
            }

            if let Some(umc) = res.borrow_components::<UiModel>().get(&entity) {
                println!("UI LOCAL - Position: {:?}, Depth: {}, Scale: {:?}", umc.position().coords, umc.depth(), umc.scale());
            }

            if let Some(sgumc) = res.borrow::<SceneGraph<UiModel>>().get(&entity) {
                println!("UI GLOBAL - Position: {:?}, Depth: {}, Scale: {:?}", sgumc.position().coords, sgumc.depth(), sgumc.scale());
            }

            let mut other_components = String::from("Other components:");
            if res.borrow_components::<Camera>().has(&entity) {
                other_components.push_str(" CAMERA");
            }

            if res.borrow_components::<Renderable>().has(&entity) {
                other_components.push_str(" RENDERABLE");
            }
            println!("{}", other_components);
        }

        if let Some(add_matches) = matches.subcommand_matches("add") {
            let add_renderable = add_matches.is_present("renderable");
            let index = add_matches.value_of("index").ok_or(Error::NoIndexSpecified)?;

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
                res.borrow_components_mut::<Renderable>().insert(entity, renderable);
            }
        }

        if let Some(create_matches) = matches.subcommand_matches("create") {
            let ui_element = create_matches.is_present("ui");
            let camera = create_matches.is_present("camera");
            let name = create_matches
                .value_of("name")
                .context("Missing required argument name")?;

            // FIXME: The following can cause issues if adding a UI entity as a child of a world entity or vice versa
            let parent = if let Some(parent_str) = create_matches.value_of("parent") {
                let parent_idx: usize = parent_str
                    .parse()
                    .context("The value of argument parent is not a positive integer")?;
                let parent_entity = res
                    .borrow::<Entities>()
                    .try_get(parent_idx)
                    .ok_or(anyhow!("The entity with index {} was not found", parent_idx))?;

                Some(parent_entity)
            } else {
                None
            };

            let new_entity = res.borrow_mut::<Entities>().create();
            res.borrow_components_mut::<Info>()
                .insert(new_entity, Info::new(name, ""));
            res.borrow_components_mut::<Status>()
                .insert(new_entity, Status::default());

            if ui_element {
                res.borrow_components_mut::<UiModel>()
                    .insert(new_entity, UiModel::default());
                if let Some(ref p) = parent {
                    res.borrow_mut::<SceneGraph<UiModel>>().insert_child(p, new_entity);
                } else {
                    res.borrow_mut::<SceneGraph<UiModel>>().insert(new_entity);
                }
            } else {
                if camera {
                    res.borrow_components_mut::<Camera>()
                        .insert(new_entity, Camera::default());
                }
                res.borrow_components_mut::<Model>()
                    .insert(new_entity, Model::default());
                if let Some(ref p) = parent {
                    res.borrow_mut::<SceneGraph<Model>>().insert_child(p, new_entity);
                } else {
                    res.borrow_mut::<SceneGraph<Model>>().insert(new_entity);
                }
            }
        }

        if let Some(list_matches) = matches.subcommand_matches("list") {
            let show_count = list_matches.is_present("count");
            let show_disabled = list_matches.is_present("disabled");
            let show_hidden = list_matches.is_present("hidden");

            let entities = res.borrow::<Entities>();
            let infos = res.borrow_components::<Info>();
            let statuses = res.borrow_components::<Status>();

            if show_count {
                println!("Loaded entities (including disabled and hidden): {}", entities.len());
            }

            for entity in &*entities {
                if statuses.get(entity).map_or(true, |s| !((show_disabled || s.enabled()) && (show_hidden || s.visible()))) {
                    continue
                }
                let (name, description) = infos.get(entity).map_or(("(no name)", "(no description)"), |i| (i.name(), i.description()));
                println!("{} {} {}", entity.idx(), name, description);
            }
        }

        if let Some(status_matches) = matches.subcommand_matches("status") {
            let entities = res.borrow::<Entities>();
            let mut statuses = res.borrow_components_mut::<Status>();
            let index = status_matches.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = entities.try_get(index).ok_or(Error::EntityNotFound(index))?;

            let enabled = if status_matches.is_present("enable") {
                Some(true)
            } else if status_matches.is_present("disable") {
                Some(false)
            } else {
                None
            };

            let visible = if status_matches.is_present("show") {
                Some(true)
            } else if status_matches.is_present("hide") {
                Some(false)
            } else {
                None
            };

            res.borrow_mut::<EventQueue<EngineEvent>>()
                .send(EngineEvent::SetStatus { entity, enabled, visible });
        }

        Ok(())
    }
}
