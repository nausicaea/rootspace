use std::collections::HashMap;

use super::ids::{BindGroupId, BindGroupLayoutId, BufferId, PipelineId, SamplerId, TextureId};

#[derive(Debug, Default)]
pub struct Tables {
    pub bind_group_layouts: HashMap<BindGroupLayoutId, wgpu::BindGroupLayout>,
    pub bind_groups: HashMap<BindGroupId, wgpu::BindGroup>,
    pub buffers: HashMap<BufferId, wgpu::Buffer>,
    pub textures: HashMap<TextureId, (wgpu::Texture, wgpu::TextureView)>,
    pub samplers: HashMap<SamplerId, wgpu::Sampler>,
    pub render_pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
}
