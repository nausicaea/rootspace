use super::{gpu_material::GpuMaterial, gpu_mesh::GpuMesh};

#[derive(Debug)]
pub struct GpuModel {
    pub mesh: GpuMesh,
    pub materials: Vec<GpuMaterial>,
}
