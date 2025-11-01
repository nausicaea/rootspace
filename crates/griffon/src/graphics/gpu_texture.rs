use super::ids::{SamplerId, TextureId, TextureViewId};

#[derive(Debug, Clone)]
pub struct GpuTexture {
    pub texture: TextureId,
    pub view: TextureViewId,
    pub sampler: SamplerId,
}
