use crate::{
    engine::resources::graphics::{
        ids::{SamplerId, TextureId, TextureViewId},
    },
};

#[derive(Debug)]
pub struct GpuTexture {
    pub texture: TextureId,
    pub view: TextureViewId,
    pub sampler: SamplerId,
}