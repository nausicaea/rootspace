use ecs::{with_dependencies::WithDependencies, Resource};
use pollster::FutureExt;
use winit::event_loop::EventLoopWindowTarget;

use self::{
    bind_group_builder::BindGroupBuilder,
    bind_group_layout_builder::BindGroupLayoutBuilder,
    ids::{BindGroupId, BindGroupLayoutId, BufferId, ShaderModuleId, TextureId, TextureViewId},
    indexes::Indexes,
    render_pass_builder::RenderPassBuilder,
    render_pipeline_builder::RenderPipelineBuilder,
    runtime::Runtime,
    sampler_builder::SamplerBuilder,
    settings::Settings,
    tables::Tables,
    texture_builder::TextureBuilder,
};

pub mod bind_group_builder;
pub mod bind_group_layout_builder;
pub mod descriptors;
pub mod ids;
mod indexes;
pub mod render_pass_builder;
pub mod render_pipeline_builder;
mod runtime;
pub mod sampler_builder;
pub mod settings;
mod tables;
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
    indexes: Indexes,
    tables: Tables,
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

    pub fn buffer(&self, buf: &BufferId) -> &wgpu::Buffer {
        &self.tables.buffers[buf]
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

        let id = self.indexes.shader_modules.take();
        self.tables.shader_modules.insert(id, sm);
        id
    }

    pub fn create_render_pass(&self) -> RenderPassBuilder {
        RenderPassBuilder::new(&self.runtime, &self.settings, &self.tables)
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        RenderPipelineBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables)
    }

    pub fn create_bind_group_layout(&mut self) -> BindGroupLayoutBuilder {
        BindGroupLayoutBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables)
    }

    pub fn create_bind_group(&mut self, layout: BindGroupLayoutId) -> BindGroupBuilder {
        BindGroupBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables, layout)
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

        let id = self.indexes.buffers.take();
        self.tables.buffers.insert(id, buf);
        id
    }

    pub fn create_texture(&mut self) -> TextureBuilder {
        TextureBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables)
    }

    pub fn create_texture_view(&mut self, texture: TextureId) -> TextureViewId {
        let texture = &self.tables.textures[&texture];
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id = self.indexes.texture_views.take();
        self.tables.texture_views.insert(id, view);
        id
    }

    pub fn create_sampler(&mut self) -> SamplerBuilder {
        SamplerBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables)
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

        Ok(Graphics {
            settings: settings.clone(),
            runtime,
            indexes: Indexes::default(),
            tables: Tables::default(),
        })
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
