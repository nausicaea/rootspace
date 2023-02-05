use crate::resources::{asset_database::AssetDatabase, graphics::Graphics};

use super::{material::Material, mesh::Mesh, Asset, Error};

#[derive(Debug)]
pub struct Model {
    pub mesh: Mesh,
    pub materials: Vec<Material>,
}

impl Model {
    pub(crate) fn with_ply(adb: &AssetDatabase, gfx: &mut Graphics, ply: &plyers::Ply) -> Result<Self, Error> {
        let texture_file_names = ply
            .descriptor
            .comments
            .iter()
            .chain(ply.descriptor.elements.values().flat_map(|e| e.comments.iter()))
            .chain(
                ply.descriptor
                    .elements
                    .values()
                    .flat_map(|e| e.properties.values().flat_map(|p| p.comments())),
            )
            .map(|c| AsRef::<str>::as_ref(c))
            .filter(|c| c.starts_with("TextureFile"))
            .map(|c| c.trim_start_matches("TextureFile "))
            .chain(
                ply.descriptor
                    .obj_info
                    .iter()
                    .chain(ply.descriptor.elements.values().flat_map(|e| e.obj_info.iter()))
                    .chain(
                        ply.descriptor
                            .elements
                            .values()
                            .flat_map(|e| e.properties.values().flat_map(|p| p.obj_info())),
                    )
                    .map(|c| AsRef::<str>::as_ref(c))
                    .filter(|c| c.starts_with("texture"))
                    .map(|c| c.trim_start_matches("texture ")),
            )
            .collect::<Vec<&str>>();

        log::trace!("Located the following texture names: {}", texture_file_names.join(", "));

        let mut materials = Vec::new();
        for name in texture_file_names {
            let texture_path = adb.find_asset("textures", name)?;
            materials.push(Material::with_file(gfx, &texture_path)?);
        }

        Ok(Model {
            mesh: Mesh::with_ply(gfx, ply)?,
            materials,
        })
    }
}

impl Asset for Model {
    type Error = Error;

    fn with_file<S: AsRef<str>>(
        adb: &AssetDatabase,
        gfx: &mut Graphics,
        group: S,
        name: S,
    ) -> Result<Self, Self::Error> {
        let path = adb.find_asset(group, name)?;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("ply") => {
                let ply = plyers::load_ply(path)?;
                Self::with_ply(adb, gfx, &ply)
            }
            _ => Err(Error::UnsupportedFileFormat),
        }
    }
}
