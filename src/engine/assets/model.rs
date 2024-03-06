use std::path::Path;

use anyhow::Context;

use crate::ecs::resources::Resources;
use crate::engine::assets::raw_mesh::CpuMesh;
use crate::engine::resources::asset_database::AssetDatabase;
use crate::plyers;
use crate::plyers::types::Ply;

use super::{material::Material, mesh::GpuMesh, private::PrivLoadAsset, Error};

#[derive(Debug)]
pub struct Model {
    pub mesh: GpuMesh,
    pub materials: Vec<Material>,
}

impl Model {
    pub(crate) async fn with_ply(res: &Resources, ply: &Ply, material_group: &str) -> Result<Self, anyhow::Error> {
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
            .map(AsRef::<str>::as_ref)
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
                    .map(AsRef::<str>::as_ref)
                    .filter(|c| c.starts_with("texture"))
                    .map(|c| c.trim_start_matches("texture ")),
            )
            .collect::<Vec<&str>>();

        log::trace!("Located the following texture names: {}", texture_file_names.join(", "));

        let mut materials = Vec::new();
        for name in texture_file_names {
            let path = res
                .read::<AssetDatabase>()
                .find_asset(material_group, name)
                .with_context(|| {
                    format!(
                        "trying to find the material asset '{}' in group '{}'",
                        name, material_group
                    )
                })?;
            let material = Material::with_path(res, &path).await.with_context(|| {
                format!(
                    "trying to load a {} from path '{}'",
                    std::any::type_name::<Material>(),
                    path.display()
                )
            })?;
            materials.push(material);
        }

        let raw_mesh = CpuMesh::with_ply(ply).context("trying to load a raw mesh from Stanford Ply data")?;

        let mesh = GpuMesh::with_raw_mesh(res, &raw_mesh)
            .context("trying to load a GPU-native mesh from the raw mesh data")?;

        Ok(Model { mesh, materials })
    }
}

impl PrivLoadAsset for Model {
    type Output = Self;

    async fn with_path(res: &Resources, path: &Path) -> Result<Self::Output, anyhow::Error> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("ply") => {
                let ply = plyers::load_ply(path)
                    .with_context(|| format!("trying to load a Stanford Ply file from '{}'", path.display()))?;
                Self::with_ply(res, &ply, "textures").await
            }
            _ => Err(Error::UnsupportedFileFormat.into()),
        }
    }
}
