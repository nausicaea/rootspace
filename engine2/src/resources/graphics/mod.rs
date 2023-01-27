use std::collections::HashMap;

use ecs::{with_dependencies::WithDependencies, Resource};
use pollster::FutureExt;
use urn::Urn;
use winit::event_loop::EventLoopWindowTarget;

use self::{
    bind_group_builder::BindGroupBuilder,
    bind_group_layout_builder::BindGroupLayoutBuilder,
    encoder::Encoder,
    ids::{BindGroupId, BindGroupLayoutId, BufferId, PipelineId, SamplerId, ShaderModuleId, TextureId, TextureViewId},
    render_pipeline_builder::RenderPipelineBuilder,
    runtime::Runtime,
    sampler_builder::SamplerBuilder,
    settings::Settings,
    texture_builder::TextureBuilder,
};

pub mod bind_group_builder;
pub mod bind_group_layout_builder;
pub mod descriptors;
pub mod encoder;
pub mod ids;
pub mod render_pipeline_builder;
mod runtime;
pub mod sampler_builder;
pub mod settings;
pub mod texture_builder;
pub mod vertex;

pub trait GraphicsDeps {
    type CustomEvent: 'static;

    fn event_loop(&self) -> &EventLoopWindowTarget<Self::CustomEvent>;
    fn settings(&self) -> &Settings;
}

#[derive(Debug)]
pub struct Graphics {
    settings: Settings,
    runtime: Runtime,
    database: Database,
    transform_layout: BindGroupLayoutId,
    material_layout: BindGroupLayoutId,
}

impl Graphics {
    pub fn window_id(&self) -> winit::window::WindowId {
        self.runtime.window.id()
    }

    pub fn reconfigure(&mut self) {
        self.resize(self.runtime.size)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.runtime.size = new_size;
            self.runtime.config.width = new_size.width;
            self.runtime.config.height = new_size.height;
            self.runtime
                .surface
                .configure(&self.runtime.device, &self.runtime.config);
        }
    }

    pub fn transform_layout(&self) -> BindGroupLayoutId {
        self.transform_layout
    }

    pub fn material_layout(&self) -> BindGroupLayoutId {
        self.material_layout
    }

    pub fn create_shader_module<'s, S: Into<std::borrow::Cow<'s, str>>>(
        &mut self,
        label: Option<&str>,
        source: S,
    ) -> ShaderModuleId {
        let sm = self.runtime.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.database.insert_shader_module(None, sm)
    }

    pub fn create_encoder(&self) -> Result<Encoder, wgpu::SurfaceError> {
        Encoder::new(&self.runtime, &self.settings, &self.database)
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        RenderPipelineBuilder::new(&self.runtime, &mut self.database)
    }

    pub fn create_bind_group_layout(&mut self) -> BindGroupLayoutBuilder {
        BindGroupLayoutBuilder::new(&self.runtime, &mut self.database)
    }

    pub fn create_bind_group(&mut self, layout: BindGroupLayoutId) -> BindGroupBuilder {
        BindGroupBuilder::new(&self.runtime, &mut self.database, layout)
    }

    pub fn create_buffer<T: bytemuck::NoUninit>(
        &mut self,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        contents: &[T],
    ) -> BufferId {
        use wgpu::util::DeviceExt;

        let buf = self
            .runtime
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                usage,
                contents: bytemuck::cast_slice(contents),
            });

        self.database.insert_buffer(None, buf)
    }

    pub fn create_texture(&mut self) -> TextureBuilder {
        TextureBuilder::new(&self.runtime, &mut self.database)
    }

    pub fn create_texture_view(&mut self, texture: TextureId) -> TextureViewId {
        let texture = &self.database.textures[&texture];
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.database.insert_texture_view(None, view)
    }

    pub fn create_sampler(&mut self) -> SamplerBuilder {
        SamplerBuilder::new(&self.runtime, &mut self.database)
    }
}

impl Resource for Graphics {}

impl<D: GraphicsDeps> WithDependencies<D> for Graphics {
    fn with_deps(deps: &D) -> Result<Self, anyhow::Error> {
        let settings = deps.settings();
        let runtime = Runtime::new(
            deps.event_loop(),
            settings.backends,
            settings.power_preference,
            settings.features,
            settings.limits.clone(),
            &settings.preferred_texture_format,
            settings.present_mode,
            settings.alpha_mode,
        )
        .block_on();

        let mut database = Database::default();

        let transform_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .add_bind_group_layout_entry(
                0,
                wgpu::ShaderStages::VERTEX,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .submit(Some("transform_layout"));

        let material_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .add_bind_group_layout_entry(
                0,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
            )
            .add_bind_group_layout_entry(
                1,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            )
            .submit(Some("material_layout"));

        Ok(Graphics {
            settings: settings.clone(),
            runtime,
            database,
            transform_layout,
            material_layout,
        })
    }
}

#[derive(Debug, Default)]
pub struct Database {
    shader_module_index: Urn<ShaderModuleId>,
    shader_modules: HashMap<ShaderModuleId, wgpu::ShaderModule>,
    bind_group_layout_index: Urn<BindGroupLayoutId>,
    bind_group_layouts: HashMap<BindGroupLayoutId, wgpu::BindGroupLayout>,
    bind_group_index: Urn<BindGroupId>,
    bind_groups: HashMap<BindGroupId, wgpu::BindGroup>,
    buffer_index: Urn<BufferId>,
    buffers: HashMap<BufferId, wgpu::Buffer>,
    texture_index: Urn<TextureId>,
    textures: HashMap<TextureId, wgpu::Texture>,
    texture_view_index: Urn<TextureViewId>,
    texture_views: HashMap<TextureViewId, wgpu::TextureView>,
    sampler_index: Urn<SamplerId>,
    samplers: HashMap<SamplerId, wgpu::Sampler>,
    render_pipeline_index: Urn<PipelineId>,
    render_pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
}

impl Database {
    fn insert_shader_module(&mut self, label: Option<&'static str>, obj: wgpu::ShaderModule) -> ShaderModuleId {
        let id = self.shader_module_index.take_labelled(label);
        self.shader_modules.insert(id, obj);
        id
    }

    fn insert_bind_group_layout(
        &mut self,
        label: Option<&'static str>,
        obj: wgpu::BindGroupLayout,
    ) -> BindGroupLayoutId {
        let id = self.bind_group_layout_index.take_labelled(label);
        self.bind_group_layouts.insert(id, obj);
        id
    }

    fn insert_bind_group(&mut self, label: Option<&'static str>, obj: wgpu::BindGroup) -> BindGroupId {
        let id = self.bind_group_index.take_labelled(label);
        self.bind_groups.insert(id, obj);
        id
    }

    fn insert_buffer(&mut self, label: Option<&'static str>, obj: wgpu::Buffer) -> BufferId {
        let id = self.buffer_index.take_labelled(label);
        self.buffers.insert(id, obj);
        id
    }

    fn insert_texture(&mut self, label: Option<&'static str>, obj: wgpu::Texture) -> TextureId {
        let id = self.texture_index.take_labelled(label);
        self.textures.insert(id, obj);
        id
    }

    fn insert_texture_view(&mut self, label: Option<&'static str>, obj: wgpu::TextureView) -> TextureViewId {
        let id = self.texture_view_index.take_labelled(label);
        self.texture_views.insert(id, obj);
        id
    }

    fn insert_sampler(&mut self, label: Option<&'static str>, obj: wgpu::Sampler) -> SamplerId {
        let id = self.sampler_index.take_labelled(label);
        self.samplers.insert(id, obj);
        id
    }

    fn insert_render_pipeline(&mut self, label: Option<&'static str>, obj: wgpu::RenderPipeline) -> PipelineId {
        let id = self.render_pipeline_index.take_labelled(label);
        self.render_pipelines.insert(id, obj);
        id
    }
}

#[cfg(test)]
mod tests {
    use ecs::Reg;

    use super::*;

    #[test]
    fn graphics_reg_macro() {
        type _RR = Reg![Graphics];
    }
}
