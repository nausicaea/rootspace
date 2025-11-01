use super::gpu_texture::GpuTexture;
use super::ids::BindGroupId;

#[derive(Debug, Clone)]
pub struct GpuMaterial {
    pub texture: GpuTexture,
    pub bind_group: BindGroupId,
}
