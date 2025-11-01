use crate::base::gpu_texture::GpuTexture;
use crate::base::ids::BindGroupId;

#[derive(Debug, Clone)]
pub struct GpuMaterial {
    pub texture: GpuTexture,
    pub bind_group: BindGroupId,
}
