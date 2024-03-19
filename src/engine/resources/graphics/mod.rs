use winit::event_loop::EventLoopWindowTarget;

use self::{
    bind_group_builder::BindGroupBuilder,
    bind_group_layout_builder::BindGroupLayoutBuilder,
    database::Database,
    encoder::Encoder,
    ids::{BindGroupLayoutId, BufferId, ShaderModuleId, TextureId, TextureViewId},
    render_pipeline_builder::RenderPipelineBuilder,
    runtime::Runtime,
    sampler_builder::SamplerBuilder,
    settings::Settings,
    texture_builder::TextureBuilder,
};
use crate::{
    ecs::{resource::Resource, with_dependencies::WithDependencies},
    glamour::mat::Mat4,
};

pub mod bind_group_builder;
pub mod bind_group_layout_builder;
mod database;
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
    runtime: Runtime<'static>,
    database: Database,
    transform_layout: BindGroupLayoutId,
    material_layout: BindGroupLayoutId,
}

impl Graphics {
    pub fn max_window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.runtime.max_size
    }

    pub fn max_objects(&self) -> u32 {
        self.settings.max_objects
    }

    pub fn limits(&self) -> wgpu::Limits {
        self.runtime.device.limits()
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.runtime.window.id()
    }

    pub fn request_redraw(&self) {
        self.runtime.window.request_redraw()
    }

    pub fn reconfigure(&mut self) {
        self.resize(self.runtime.size)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > self.runtime.max_size.width || new_size.height > self.runtime.max_size.height {
            tracing::warn!(
                "Ignoring requested physical dimensions {}x{} because they exceed maximum dimensions {}x{}",
                new_size.width,
                new_size.height,
                self.runtime.max_size.width,
                self.runtime.max_size.height
            );
            return;
        }

        self.runtime.size = new_size;
        self.runtime.config.width = new_size.width;
        self.runtime.config.height = new_size.height;
        self.runtime
            .surface
            .configure(&self.runtime.device, &self.runtime.config);
    }

    pub fn transform_layout(&self) -> BindGroupLayoutId {
        self.transform_layout
    }

    pub fn material_layout(&self) -> BindGroupLayoutId {
        self.material_layout
    }

    pub fn write_buffer<T>(&self, buffer: BufferId, data: &[T])
    where
        T: bytemuck::NoUninit,
    {
        self.runtime
            .queue
            .write_buffer(&self.database.buffers[&buffer], 0, bytemuck::cast_slice(data));
    }

    pub fn create_shader_module<'a, 's, S: Into<std::borrow::Cow<'s, str>>>(
        &mut self,
        label: Option<&'a str>,
        source: S,
    ) -> ShaderModuleId {
        tracing::trace!("Creating shader module '{}'", label.unwrap_or("unnamed"));
        let sm = self.runtime.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.database.insert_shader_module(sm)
    }

    pub fn create_encoder(&self, label: Option<&str>) -> Result<Encoder, wgpu::SurfaceError> {
        Encoder::new(label, &self.runtime, &self.settings, &self.database)
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        RenderPipelineBuilder::new(&self.runtime, &mut self.database)
    }

    pub fn create_bind_group(&mut self, layout: BindGroupLayoutId) -> BindGroupBuilder {
        BindGroupBuilder::new(&self.runtime, &mut self.database, layout)
    }

    pub fn create_buffer(
        &mut self,
        label: Option<&str>,
        size: wgpu::BufferAddress,
        usage: wgpu::BufferUsages,
    ) -> BufferId {
        tracing::trace!("Creating buffer '{}'", label.unwrap_or("unnamed"));
        let buf = self.runtime.device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage,
            mapped_at_creation: false,
        });

        self.database.insert_buffer(buf)
    }

    pub fn create_buffer_init<T: bytemuck::NoUninit>(
        &mut self,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        contents: &[T],
    ) -> BufferId {
        use wgpu::util::DeviceExt;

        tracing::trace!("Creating and initializing buffer '{}'", label.unwrap_or("unnamed"));
        let buf = self
            .runtime
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                usage,
                contents: bytemuck::cast_slice(contents),
            });

        self.database.insert_buffer(buf)
    }

    pub fn create_texture(&mut self) -> TextureBuilder {
        TextureBuilder::new(&self.runtime, &mut self.database)
    }

    pub fn create_texture_view(&mut self, label: Option<&str>, texture: TextureId) -> TextureViewId {
        let texture = &self.database.textures[&texture];

        tracing::trace!("Creating texture view '{}'", label.unwrap_or("unnamed"));
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            ..Default::default()
        });
        self.database.insert_texture_view(view)
    }

    pub fn create_sampler(&mut self) -> SamplerBuilder {
        SamplerBuilder::new(&self.runtime, &mut self.database)
    }
}

impl Resource for Graphics {}

impl<D> WithDependencies<D> for Graphics
where
    D: GraphicsDeps + std::fmt::Debug,
{
    #[tracing::instrument(skip_all)]
    async fn with_deps(deps: &D) -> Result<Self, anyhow::Error> {
        let settings = deps.settings();
        let runtime = Runtime::new(deps.event_loop(), settings).await;

        let mut database = Database::default();

        let min_binding_size = wgpu::BufferSize::new(std::mem::size_of::<TransformWrapper>() as _); // 64 bytes
        let transform_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("transform-layout")
            .add_bind_group_layout_entry(
                0,
                wgpu::ShaderStages::VERTEX,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size,
                },
            )
            .submit();

        let material_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("material-layout")
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
            .submit();

        Ok(Graphics {
            settings: settings.clone(),
            runtime,
            database,
            transform_layout,
            material_layout,
        })
    }
}

#[repr(C, align(256))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformWrapper(pub Mat4<f32>);

impl From<Mat4<f32>> for TransformWrapper {
    fn from(value: Mat4<f32>) -> Self {
        TransformWrapper(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Reg;

    #[test]
    fn graphics_reg_macro() {
        type _RR = Reg![Graphics];
    }
}
