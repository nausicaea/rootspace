use crate::base::ids::{BindGroupLayoutId, InstanceId, TextureId, TextureViewId};
use urn::Urn;

#[derive(Debug)]
pub struct InternalRuntimeData {
    pub camera_bind_group_layout: BindGroupLayoutId,
    pub light_bind_group_layout: BindGroupLayoutId,
    pub material_bind_group_layout: BindGroupLayoutId,
    pub depth_texture: TextureId,
    pub depth_texture_view: TextureViewId,
    pub instances: Urn<InstanceId>,
}
