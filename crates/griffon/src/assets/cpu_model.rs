use std::path::Path;

use anyhow::Context;

use super::{cpu_material::CpuMaterial, cpu_mesh::CpuMesh};
use assam::{AssetDatabase, LoadAsset};
use ecs::resources::Resources;

pub const MATERIAL_ASSET_GROUP: &str = "textures";

#[derive(Debug)]
pub struct CpuModel {
    pub mesh: CpuMesh,
    pub materials: Vec<CpuMaterial>,
}

impl LoadAsset for CpuModel {
    type Output = Self;

    async fn with_path(res: &Resources, path: &Path) -> Result<Self::Output, anyhow::Error> {
        let mesh = CpuMesh::with_path(res, path)
            .await
            .with_context(|| format!("Loading a CpuMesh from '{}'", path.display()))?;
        tracing::trace!("Loaded CpuMesh with size {} bytes", std::mem::size_of_val(&mesh));

        let mut materials = Vec::new();
        for name in &mesh.texture_names {
            let cpu_mat = res
                .read::<AssetDatabase>()
                .load_asset::<CpuMaterial, _>(res, MATERIAL_ASSET_GROUP, name)
                .await
                .with_context(|| {
                    format!(
                        "Loading a CpuMaterial from group {} and name {}",
                        MATERIAL_ASSET_GROUP, name
                    )
                })?;
            tracing::trace!("Loaded CpuMaterial with size {} bytes", std::mem::size_of_val(&cpu_mat));

            materials.push(cpu_mat);
        }

        Ok(CpuModel { mesh, materials })
    }
}
