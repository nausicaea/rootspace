use super::ids::{BindGroupLayoutId, InstanceId, TextureId, TextureViewId};
use urn::Urn;

#[derive(Debug)]
pub struct InternalRuntimeData {
    pub camera_buffer_layout: BindGroupLayoutId,
    pub light_buffer_layout: BindGroupLayoutId,
    pub material_buffer_layout: BindGroupLayoutId,
    pub depth_texture: TextureId,
    pub depth_texture_view: TextureViewId,
    pub instances: Urn<InstanceId>,
}
