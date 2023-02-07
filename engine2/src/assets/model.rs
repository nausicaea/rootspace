use std::path::Path;

use ecs::Resources;

use crate::{assets::raw_mesh::RawMesh, resources::asset_database::AssetDatabase};

use super::{material::Material, mesh::Mesh, Asset, Error};

#[derive(Debug)]
pub struct Model {
    pub mesh: Mesh,
    pub materials: Vec<Material>,
}

impl Model {
    pub(crate) fn with_ply(res: &Resources, ply: &plyers::Ply, material_group: &str) -> Result<Self, Error> {
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
            let path = res.borrow::<AssetDatabase>().find_asset(material_group, name)?;
            materials.push(Material::with_path(res, &path)?);
        }

        let raw_mesh = RawMesh::with_ply(ply)?;

        Ok(Model {
            mesh: Mesh::with_raw_mesh(res, &raw_mesh)?,
            materials,
        })
    }
}

impl Asset for Model {
    type Output = Self;

    fn with_path(res: &Resources, path: &Path) -> Result<Self::Output, Error> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("ply") => {
                let ply = plyers::load_ply(path)?;
                Self::with_ply(res, &ply, "textures")
            }
            _ => Err(Error::UnsupportedFileFormat),
        }
    }
}
