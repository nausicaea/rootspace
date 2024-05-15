use crate::urn::Urn;
use super::ids::{BindGroupLayoutId, InstanceId, TextureId, TextureViewId};

#[derive(Debug)]
pub struct InternalRuntimeData {
    pub transform_layout: BindGroupLayoutId,
    pub material_layout: BindGroupLayoutId,
    pub depth_texture: TextureId,
    pub depth_texture_view: TextureViewId,
    pub instances: Urn<InstanceId>,
}
