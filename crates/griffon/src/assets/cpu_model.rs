use std::path::Path;

use super::{cpu_material::CpuMaterial, cpu_mesh::CpuMesh};
use anyhow::Context;
use assam::{AssetDatabase, LoadAsset};
use ecs::Resources;
use tracing::trace;

pub const MATERIAL_ASSET_GROUP: &str = "textures";

#[derive(Debug)]
pub struct CpuModel {
    pub mesh: CpuMesh,
    pub materials: Vec<CpuMaterial>,
}

impl LoadAsset for CpuModel {
    type Output = Self;

    fn with_path(res: &Resources, path: &Path) -> anyhow::Result<Self::Output> {
        let mesh =
            CpuMesh::with_path(res, path).with_context(|| format!("Loading a CpuMesh from '{}'", path.display()))?;
        trace!("Loaded CpuMesh with size {} bytes", size_of_val(&mesh));

        let mut materials = Vec::new();
        for name in &mesh.texture_names {
            let cpu_mat = res
                .read::<AssetDatabase>()
                .load_asset::<CpuMaterial, _>(res, MATERIAL_ASSET_GROUP, name)
                .with_context(|| {
                    format!(
                        "Loading a CpuMaterial from group {} and name {}",
                        MATERIAL_ASSET_GROUP, name
                    )
                })?;
            trace!("Loaded CpuMaterial with size {} bytes", size_of_val(&cpu_mat));

            materials.push(cpu_mat);
        }

        Ok(CpuModel { mesh, materials })
    }
}
