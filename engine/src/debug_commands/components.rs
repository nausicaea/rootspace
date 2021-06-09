use anyhow::Result;
use clap::{load_yaml, App};
use ecs::{Entities, Resources, Storage};
use serde::{Deserialize, Serialize};

use super::{CommandTrait, Error};
use crate::components::{Camera, Status};

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

        if subcommand == "status" {
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
                    .or_insert(Status::new(enable || !disable, show || !hide));
            }

            if let Some(sc) = res.borrow_components::<Status>().get(&entity) {
                let enbl = if sc.enabled() { "enabled" } else { "disabled" };
                let vis = if sc.visible() { "visible" } else { "hidden" };

                println!("Entity {}: {} {}", entity, enbl, vis);
            } else {
                println!("Entity {}: (no status)", entity);
            }
        } else if subcommand == "camera" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("camera"))?;
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .try_get(index)
                .ok_or(Error::EntityNotFound(index))?;

            let cameras = res.borrow_components::<Camera>();
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
        }

        Ok(())
    }
}
