use clap::{load_yaml, App};
use serde::{Deserialize, Serialize};

use ecs::{Entities, Resources, Storage};

use crate::{components::Camera, debug_commands::Error, CommandTrait};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CamerasCommand;

impl CommandTrait for CamerasCommand {
    fn name(&self) -> &'static str {
        "cameras"
    }

    fn description(&self) -> &'static str {
        "Provides access to the camera"
    }

    fn run(&self, res: &Resources, args: &[String]) -> anyhow::Result<()> {
        let app_yaml = load_yaml!("cameras.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;

        if let Some(info_matches) = matches.subcommand_matches("info") {
            let show_count = info_matches.is_present("count");
            let index = info_matches.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entities = res.borrow::<Entities>();
            let entity = entities.try_get(index).ok_or(Error::EntityNotFound(index))?;

            let cameras = res.borrow_components::<Camera>();

            if show_count {
                println!("Loaded cameras: {}", cameras.len());
            }

            let cam = cameras.get(entity).ok_or(Error::EntityNotFound(index))?;

            let dims = cam.dimensions();
            let pdims = cam.physical_dimensions();
            let dpi = cam.dpi_factor();

            println!("Dimensions: {}x{}", dims.0, dims.1);
            println!("Physical dimensions: {}x{}", pdims.0, pdims.1);
            println!("DPI-factor: {}", dpi);
            println!("Projection type: {}", cam.projection());
            println!("Vertical field of view: {} rad ({} deg)", cam.fov_y(), cam.fov_y() * 360.0 / std::f32::consts::PI);
            println!("Depth frustum: {:?}", cam.frustum_z());
        }

        Ok(())
    }
}
