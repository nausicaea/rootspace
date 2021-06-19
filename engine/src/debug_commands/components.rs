use std::marker::PhantomData;

use anyhow::Result;
use clap::{load_yaml, App};
use ecs::{Entities, Resources, Storage};
use serde::{Deserialize, Serialize};

use super::{CommandTrait, Error};
use crate::{
    components::{Camera, Info, Model, Renderable, RenderableType, Status, UiModel},
    graphics::BackendTrait,
    resources::{AssetDatabase, GraphicsBackend, SceneGraph},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComponentsCommand<B>(PhantomData<B>);

impl<B> ComponentsCommand<B> {
    fn info(
        &self,
        res: &Resources,
        index: &str,
        create: bool,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        let index: usize = index.parse()?;
        let entity = res
            .borrow::<Entities>()
            .get(index)
            .ok_or(Error::EntityNotFound(index))?;

        let mut infos = res.borrow_components_mut::<Info>();

        if create || name.is_some() || description.is_some() {
            infos
                .entry(entity)
                .and_modify(|i| {
                    if let Some(name) = name {
                        i.set_name(name);
                    }

                    if let Some(description) = description {
                        i.set_description(description);
                    }
                })
                .or_insert_with(|| {
                    Info::builder()
                        .with_name(name.unwrap_or(""))
                        .with_description(description.unwrap_or(""))
                        .build()
                });
        }

        if let Some(ic) = infos.get(&entity) {
            println!("Entity {}: {}", entity, ic,);
        } else {
            println!("Entity {}: (no name or description)", entity);
        }

        Ok(())
    }

    fn status(
        &self,
        res: &Resources,
        index: &str,
        create: bool,
        enable: bool,
        disable: bool,
        show: bool,
        hide: bool,
    ) -> Result<()> {
        let index: usize = index.parse()?;
        let entity = res
            .borrow::<Entities>()
            .get(index)
            .ok_or(Error::EntityNotFound(index))?;

        let mut statuses = res.borrow_components_mut::<Status>();

        if create || enable || disable || show || hide {
            statuses
                .entry(entity)
                .and_modify(|s| {
                    if enable {
                        s.enable();
                    } else if disable {
                        s.disable();
                    }

                    if show {
                        s.show();
                    } else if hide {
                        s.hide();
                    }
                })
                .or_insert_with(|| {
                    Status::builder()
                        .with_enabled(enable || !disable)
                        .with_visible(show || !hide)
                        .build()
                });
        }

        if let Some(sc) = statuses.get(&entity) {
            println!("Entity {}: {}", entity, sc);
        } else {
            println!("Entity {}: (no status)", entity);
        }

        Ok(())
    }

    fn camera(&self, res: &Resources, index: &str, create: bool) -> Result<()> {
        let index: usize = index.parse()?;
        let entity = res
            .borrow::<Entities>()
            .get(index)
            .ok_or(Error::EntityNotFound(index))?;

        let mut cameras = res.borrow_components_mut::<Camera>();

        if create {
            cameras.entry(entity).or_default();
        }

        if let Some(cc) = cameras.get(&entity) {
            let dims = cc.dimensions();
            let pdims = cc.physical_dimensions();

            println!("Dimensions: {}x{}", dims.0, dims.1);
            println!("Physical dimensions: {}x{}", pdims.0, pdims.1);
            println!("DPI-factor: {}", cc.dpi_factor());
            println!("Projection type: {}", cc.projection());
            println!(
                "Vertical field of view: {} rad ({} deg)",
                cc.fov_y(),
                cc.fov_y() * 360.0 / std::f32::consts::PI
            );
            println!("Depth frustum: {:?}", cc.frustum_z());
        } else {
            println!("Entity {}: (no camera)", entity);
        }

        Ok(())
    }
}

impl<B> Default for ComponentsCommand<B> {
    fn default() -> Self {
        ComponentsCommand(PhantomData::default())
    }
}

impl<B> CommandTrait for ComponentsCommand<B>
where
    B: BackendTrait + 'static,
{
    fn name(&self) -> &'static str {
        "components"
    }

    fn description(&self) -> &'static str {
        "Provides access to components associated with entities"
    }

    fn run(&self, res: &Resources, args: &[String]) -> Result<()> {
        let app_yaml = load_yaml!("components.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;
        let (subcommand, scm) = matches.subcommand();

        if subcommand == "info" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("info"))?;
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;
            let create = scm.is_present("create");
            let name = scm.value_of("name");
            let description = scm.value_of("description");

            self.info(res, index, create, name, description)?;
        } else if subcommand == "status" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("status"))?;
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;
            let create = scm.is_present("create");
            let enable = scm.is_present("enable");
            let disable = scm.is_present("disable");
            let show = scm.is_present("show");
            let hide = scm.is_present("hide");

            self.status(res, index, create, enable, disable, show, hide)?;
        } else if subcommand == "camera" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("camera"))?;
            let create = scm.is_present("create");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            self.camera(res, index, create)?;
        } else if subcommand == "model" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("model"))?;
            let create = scm.is_present("create");
            let parent = scm.value_of("parent");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .get(index)
                .ok_or(Error::EntityNotFound(index))?;
            let parent = if let Some(parent) = parent {
                let parent = parent.parse::<usize>()?;
                let parent = res
                    .borrow::<Entities>()
                    .get(parent)
                    .ok_or(Error::EntityNotFound(parent))?;

                Some(parent)
            } else {
                None
            };

            let mut models = res.borrow_components_mut::<Model>();

            if create || parent.is_some() {
                models.entry(entity).or_default();

                if let Some(parent) = parent {
                    res.borrow_mut::<SceneGraph<Model>>().insert_child(&parent, entity)?;
                } else {
                    res.borrow_mut::<SceneGraph<Model>>().insert(entity);
                }
            }

            if let Some(mc) = models.get(entity) {
                println!("Entity {}: {}", entity, mc);
            } else {
                println!("Entity {}: (no model)", entity);
            }
        } else if subcommand == "ui" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("ui"))?;
            let create = scm.is_present("create");
            let parent = scm.value_of("parent");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .get(index)
                .ok_or(Error::EntityNotFound(index))?;
            let parent = if let Some(parent) = parent {
                let parent = parent.parse::<usize>()?;
                let parent = res
                    .borrow::<Entities>()
                    .get(parent)
                    .ok_or(Error::EntityNotFound(parent))?;

                Some(parent)
            } else {
                None
            };

            let mut ui_models = res.borrow_components_mut::<UiModel>();

            if create || parent.is_some() {
                ui_models.entry(entity).or_default();

                if let Some(parent) = parent {
                    res.borrow_mut::<SceneGraph<UiModel>>().insert_child(&parent, entity)?;
                } else {
                    res.borrow_mut::<SceneGraph<UiModel>>().insert(entity);
                }
            }

            if let Some(umc) = ui_models.get(entity) {
                println!("Entity {}: {}", entity, umc);
            } else {
                println!("Entity {}: (no UI model)", entity);
            }
        } else if subcommand == "renderable" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("renderable"))?;
            let create = scm.is_present("create");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let mut factory = res.borrow_mut::<GraphicsBackend<B>>();
            let assets = res.borrow::<AssetDatabase>();
            let mut renderables = res.borrow_components_mut::<Renderable>();

            if create {
                renderables.entry(entity).or_insert_with(|| {
                    let font = assets
                        .find_asset("fonts/SourceSansPro-Regular.ttf")
                        .expect("Unable to find the font asset");
                    let vs = assets
                        .find_asset("shaders/text-vertex.glsl")
                        .expect("Unable to find the vertex shader asset");
                    let fs = assets
                        .find_asset("shaders/text-fragment.glsl")
                        .expect("Unable to find the fragment shader asset");

                    Renderable::builder()
                        .with_type(RenderableType::Text)
                        .with_text("Hello, World!")
                        .with_font(font)
                        .with_vertex_shader(vs)
                        .with_fragment_shader(fs)
                        .build(&mut factory)
                        .expect("Unable to create a renderable component")
                });
            }

            if let Some(_rc) = renderables.get(entity) {
                println!("Entity {}: renderable", entity);
            } else {
                println!("Entity {}: (no renderable)", entity);
            }
        }

        Ok(())
    }
}
