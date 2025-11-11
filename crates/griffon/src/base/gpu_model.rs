use crate::base::gpu_material::GpuMaterial;
use crate::base::gpu_mesh::GpuMesh;
#[derive(Debug)]
pub struct GpuModel {
    pub mesh: GpuMesh,
    pub materials: Vec<GpuMaterial>,
}
