use anyhow::Result;
use clap::{load_yaml, App};
use ecs::{Entities, Resources, Storage};
use serde::{Deserialize, Serialize};

use super::{CommandTrait, Error};
use crate::components::{Camera, Status, Info, Model, UiModel, Renderable};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComponentsCommand;

impl CommandTrait for ComponentsCommand {
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

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .try_get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let mut infos = res.borrow_components_mut::<Info>();

            if create || name.is_some() || description.is_some() {
                infos.entry(entity)
                    .and_modify(|i| {
                        if let Some(name) = name {
                            i.set_name(name);
                        }

                        if let Some(description) = description {
                            i.set_description(description);
                        }
                    })
                    .or_insert(Info::new(name.unwrap_or(""), description.unwrap_or("")));
            }

            if let Some(ic) = infos.get(&entity) {
                println!("Entity {}: Name='{}', Description='{}'", entity.idx(), ic.name(), ic.description());
            } else {
                println!("Entity {}: (no name or description)", entity.idx());
            }
        } else if subcommand == "status" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("status"))?;
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;
            let create = scm.is_present("create");
            let enable = scm.is_present("enable");
            let disable = scm.is_present("disable");
            let show = scm.is_present("show");
            let hide = scm.is_present("hide");

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .try_get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let mut statuses = res.borrow_components_mut::<Status>();

            if create || enable || disable || show || hide {
                statuses.entry(entity)
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
                    .or_insert(Status::new(enable || !disable, show || !hide));
            }

            if let Some(sc) = statuses.get(&entity) {
                let enbl = if sc.enabled() { "enabled" } else { "disabled" };
                let vis = if sc.visible() { "visible" } else { "hidden" };

                println!("Entity {}: {} {}", entity.idx(), enbl, vis);
            } else {
                println!("Entity {}: (no status)", entity.idx());
            }
        } else if subcommand == "camera" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("camera"))?;
            let create = scm.is_present("create");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .try_get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let mut cameras = res.borrow_components_mut::<Camera>();

            if !cameras.contains(&entity) && create {
                cameras.insert(entity, Camera::default());
            }

            let cam = cameras.get(entity).ok_or(Error::EntityNotFound(index))?;

            let dims = cam.dimensions();
            let pdims = cam.physical_dimensions();
            let dpi = cam.dpi_factor();

            println!("Dimensions: {}x{}", dims.0, dims.1);
            println!("Physical dimensions: {}x{}", pdims.0, pdims.1);
            println!("DPI-factor: {}", dpi);
            println!("Projection type: {}", cam.projection());
            println!(
                "Vertical field of view: {} rad ({} deg)",
                cam.fov_y(),
                cam.fov_y() * 360.0 / std::f32::consts::PI
            );
            println!("Depth frustum: {:?}", cam.frustum_z());
        } else if subcommand == "model" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("model"))?;
            let create = scm.is_present("create");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .try_get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let mut models = res.borrow_components_mut::<Model>();

            todo!()
        } else if subcommand == "ui" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("ui"))?;
            let create = scm.is_present("create");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .try_get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let mut ui_models = res.borrow_components_mut::<UiModel>();

            todo!()
        } else if subcommand == "renderable" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("renderable"))?;
            let create = scm.is_present("create");
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .try_get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let mut renderables = res.borrow_components_mut::<Renderable>();

            todo!()
        }

        Ok(())
    }
}
