use super::ids::{BindGroupLayoutId, TextureId, TextureViewId};

#[derive(Debug)]
pub struct RuntimeData {
    pub transform_layout: BindGroupLayoutId,
    pub material_layout: BindGroupLayoutId,
    pub depth_texture: TextureId,
    pub depth_texture_view: TextureViewId,
}
