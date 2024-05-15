use super::{gpu_material::GpuMaterial, gpu_mesh::GpuMesh};
use crate::ecs::resources::Resources;
use crate::engine::assets::cpu_model::CpuModel;

#[derive(Debug)]
pub struct GpuModel {
    pub mesh: GpuMesh,
    pub materials: Vec<GpuMaterial>,
}

impl GpuModel {
    pub fn with_model(res: &Resources, m: &CpuModel) -> Self {
        GpuModel {
            mesh: GpuMesh::with_mesh(res, &m.mesh),
            materials: m
                .materials
                .iter()
                .map(|mat| GpuMaterial::with_material(res, mat))
                .collect(),
        }
    }
}
