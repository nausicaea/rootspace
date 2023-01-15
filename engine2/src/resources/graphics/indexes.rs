use super::ids::{BindGroupId, BindGroupLayoutId, BufferId, PipelineId, SamplerId, TextureId, TextureViewId};
use urn::Urn;

#[derive(Debug, Default)]
pub struct Indexes {
    pub bind_group_layouts: Urn<BindGroupLayoutId>,
    pub bind_groups: Urn<BindGroupId>,
    pub buffers: Urn<BufferId>,
    pub textures: Urn<TextureId>,
    pub texture_views: Urn<TextureViewId>,
    pub samplers: Urn<SamplerId>,
    pub render_pipelines: Urn<PipelineId>,
}
