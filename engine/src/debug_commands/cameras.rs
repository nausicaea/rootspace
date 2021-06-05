use clap::{load_yaml, App, ArgMatches};
use serde::{Deserialize, Serialize};

use ecs::{Entities, Entity, Resources, Storage};

use crate::{components::Camera, debug_commands::Error, CommandTrait};

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

    fn run(&self, res: &Resources, args: &[String]) -> anyhow::Result<()> {
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
