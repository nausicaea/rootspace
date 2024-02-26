use crate::urn::Urn;
use std::collections::HashMap;

use super::ids::{
    BindGroupId, BindGroupLayoutId, BufferId, PipelineId, SamplerId, ShaderModuleId, TextureId, TextureViewId,
};

#[derive(Debug, Default)]
pub struct Database {
    pub shader_module_index: Urn<ShaderModuleId>,
    pub shader_modules: HashMap<ShaderModuleId, wgpu::ShaderModule>,
    pub bind_group_layout_index: Urn<BindGroupLayoutId>,
    pub bind_group_layouts: HashMap<BindGroupLayoutId, wgpu::BindGroupLayout>,
    pub bind_group_index: Urn<BindGroupId>,
    pub bind_groups: HashMap<BindGroupId, wgpu::BindGroup>,
    pub buffer_index: Urn<BufferId>,
    pub buffers: HashMap<BufferId, wgpu::Buffer>,
    pub texture_index: Urn<TextureId>,
    pub textures: HashMap<TextureId, wgpu::Texture>,
    pub texture_view_index: Urn<TextureViewId>,
    pub texture_views: HashMap<TextureViewId, wgpu::TextureView>,
    pub sampler_index: Urn<SamplerId>,
    pub samplers: HashMap<SamplerId, wgpu::Sampler>,
    pub render_pipeline_index: Urn<PipelineId>,
    pub render_pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
}

impl Database {
    pub fn insert_shader_module<'a>(&mut self, obj: wgpu::ShaderModule) -> ShaderModuleId {
        let id = self.shader_module_index.take();
        self.shader_modules.insert(id, obj);
        id
    }

    pub fn insert_bind_group_layout<'a>(&mut self, obj: wgpu::BindGroupLayout) -> BindGroupLayoutId {
        let id = self.bind_group_layout_index.take();
        self.bind_group_layouts.insert(id, obj);
        id
    }

    pub fn insert_bind_group<'a>(&mut self, obj: wgpu::BindGroup) -> BindGroupId {
        let id = self.bind_group_index.take();
        self.bind_groups.insert(id, obj);
        id
    }

    pub fn insert_buffer<'a>(&mut self, obj: wgpu::Buffer) -> BufferId {
        let id = self.buffer_index.take();
        self.buffers.insert(id, obj);
        id
    }

    pub fn insert_texture<'a>(&mut self, obj: wgpu::Texture) -> TextureId {
        let id = self.texture_index.take();
        self.textures.insert(id, obj);
        id
    }

    pub fn insert_texture_view<'a>(&mut self, obj: wgpu::TextureView) -> TextureViewId {
        let id = self.texture_view_index.take();
        self.texture_views.insert(id, obj);
        id
    }

    pub fn insert_sampler<'a>(&mut self, obj: wgpu::Sampler) -> SamplerId {
        let id = self.sampler_index.take();
        self.samplers.insert(id, obj);
        id
    }

    pub fn insert_render_pipeline<'a>(&mut self, obj: wgpu::RenderPipeline) -> PipelineId {
        let id = self.render_pipeline_index.take();
        self.render_pipelines.insert(id, obj);
        id
    }
}
