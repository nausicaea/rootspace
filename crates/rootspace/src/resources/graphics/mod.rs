use std::mem::size_of;

use wgpu::{BindingType, BufferAddress, BufferBindingType, BufferSize, BufferUsages, ShaderStages};
use winit::event_loop::EventLoopWindowTarget;

use self::{
    bind_group_builder::BindGroupBuilder,
    bind_group_layout_builder::BindGroupLayoutBuilder,
    camera_uniform::CameraUniform,
    encoder::Encoder,
    gpu_object_database::GpuObjectDatabase,
    ids::{BindGroupLayoutId, BufferId, ShaderModuleId, TextureId, TextureViewId},
    light_uniform::LightUniform,
    render_pipeline_builder::RenderPipelineBuilder,
    runtime::Runtime,
    sampler_builder::SamplerBuilder,
    settings::Settings,
    texture_builder::TextureBuilder,
};
use crate::assets::cpu_material::CpuMaterial;
use crate::assets::cpu_mesh::CpuMesh;
use crate::assets::cpu_model::CpuModel;
use crate::assets::cpu_texture::CpuTexture;
use crate::resources::graphics::gpu_material::GpuMaterial;
use crate::resources::graphics::gpu_mesh::GpuMesh;
use crate::resources::graphics::gpu_model::GpuModel;
use crate::resources::graphics::gpu_texture::GpuTexture;
use crate::resources::graphics::instance::Instance;
use crate::resources::graphics::internal_runtime_data::InternalRuntimeData;
use ecs::{resource::Resource, with_dependencies::WithDependencies};
use urn::Urn;

pub mod bind_group_builder;
pub mod bind_group_layout_builder;
pub mod camera_uniform;
pub mod descriptors;
pub mod encoder;
pub mod gpu_material;
pub mod gpu_mesh;
pub mod gpu_model;
mod gpu_object_database;
pub mod gpu_texture;
pub mod ids;
pub mod instance;
mod internal_runtime_data;
pub mod light_uniform;
pub mod render_pipeline_builder;
mod runtime;
pub mod sampler_builder;
pub mod settings;
pub mod texture_builder;
pub mod vertex;

const DEPTH_TEXTURE_LABEL: Option<&str> = Some("depth-stencil:texture");
const DEPTH_TEXTURE_VIEW_LABEL: Option<&str> = Some("depth-stencil:view");

pub trait GraphicsDeps {
    type CustomEvent: 'static;

    fn event_loop(&self) -> &EventLoopWindowTarget<Self::CustomEvent>;
    fn settings(&self) -> &Settings;
}

#[derive(Debug)]
pub struct Graphics {
    settings: Settings,
    runtime: Runtime<'static>,
    database: GpuObjectDatabase,
    internal: InternalRuntimeData,
}

impl Graphics {
    pub fn max_window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.runtime.max_size
    }

    pub fn max_cameras(&self) -> u32 {
        self.settings.max_cameras
    }

    pub fn max_lights(&self) -> u32 {
        self.settings.max_lights
    }

    pub fn max_instances(&self) -> u64 {
        self.settings.max_instances
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

        self.internal.depth_texture =
            Self::create_depth_texture_int(&self.runtime, &mut self.database, &self.settings, DEPTH_TEXTURE_LABEL);
        self.internal.depth_texture_view = Self::create_texture_view_int(
            &mut self.database,
            DEPTH_TEXTURE_VIEW_LABEL,
            self.internal.depth_texture,
        );
    }

    pub fn camera_buffer_layout(&self) -> BindGroupLayoutId {
        self.internal.camera_buffer_layout
    }

    pub fn light_buffer_layout(&self) -> BindGroupLayoutId {
        self.internal.light_buffer_layout
    }

    pub fn material_buffer_layout(&self) -> BindGroupLayoutId {
        self.internal.material_buffer_layout
    }

    pub fn write_buffer<T>(&self, buffer: BufferId, data: &[T])
    where
        T: bytemuck::NoUninit,
    {
        self.runtime
            .queue
            .write_buffer(&self.database.buffers[&buffer], 0, bytemuck::cast_slice(data));
    }

    #[must_use]
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
        Encoder::new(
            label,
            &self.runtime,
            &self.settings,
            &self.database,
            self.internal.depth_texture_view,
        )
    }

    #[must_use]
    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        RenderPipelineBuilder::new(&self.runtime, &mut self.database, &self.settings)
    }

    #[must_use]
    pub fn create_bind_group(&mut self, layout: BindGroupLayoutId) -> BindGroupBuilder {
        BindGroupBuilder::new(&self.runtime, &mut self.database, layout)
    }

    #[must_use]
    pub fn create_buffer(&mut self, label: Option<&str>, size: BufferAddress, usage: BufferUsages) -> BufferId {
        tracing::trace!("Creating buffer '{}'", label.unwrap_or("unnamed"));
        let buf = self.runtime.device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage,
            mapped_at_creation: false,
        });

        self.database.insert_buffer(buf)
    }

    #[must_use]
    pub fn create_buffer_init<T: bytemuck::NoUninit>(
        &mut self,
        label: Option<&str>,
        usage: BufferUsages,
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

    #[tracing::instrument(skip_all)]
    #[must_use]
    fn create_texture(&mut self, t: &CpuTexture) -> GpuTexture {
        let texture = TextureBuilder::new(&self.runtime, &mut self.database, &self.settings)
            .with_label(t.label.as_ref().map(|l| format!("{}:texture", &l)).as_deref())
            .with_image(&t.image)
            .submit();
        let view = Self::create_texture_view_int(
            &mut self.database,
            t.label.as_ref().map(|l| format!("{}:texture-view", &l)).as_deref(),
            texture,
        );
        let sampler = SamplerBuilder::new(&self.runtime, &mut self.database)
            .with_label(t.label.as_ref().map(|l| format!("{}:texture-sampler", &l)).as_deref())
            .submit();

        GpuTexture { texture, view, sampler }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    fn create_material(&mut self, m: &CpuMaterial) -> GpuMaterial {
        let texture = self.create_texture(&m.texture);

        let layout = self.material_buffer_layout();
        let bind_group = self
            .create_bind_group(layout)
            .with_label(m.label.as_ref().map(|l| format!("{}:bind-group", &l)).as_deref())
            .add_texture_view(0, texture.view)
            .add_sampler(1, texture.sampler)
            .submit();

        GpuMaterial { texture, bind_group }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    fn create_mesh(&mut self, m: &CpuMesh) -> GpuMesh {
        let vertex_buffer = self.create_buffer_init(
            m.label.as_ref().map(|l| format!("{}:vertex-buffer", &l)).as_deref(),
            BufferUsages::VERTEX,
            &m.vertices,
        );
        let instance_buffer = {
            let max_instances = self.max_instances();
            let buffer_alignment = size_of::<Instance>() as u64;
            let buffer_size = (max_instances * buffer_alignment) as BufferAddress;
            self.create_buffer(
                m.label.as_ref().map(|l| format!("{}:instance-buffer", &l)).as_deref(),
                buffer_size,
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            )
        };
        let index_buffer = self.create_buffer_init(
            m.label.as_ref().map(|l| format!("{}:index-buffer", &l)).as_deref(),
            BufferUsages::INDEX,
            &m.indices,
        );

        GpuMesh {
            vertex_buffer,
            instance_buffer,
            index_buffer,
            num_indices: m.indices.len() as u32,
            instance_id: self.internal.instances.take(),
        }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    fn create_instanced_mesh(&mut self, m: &GpuMesh) -> GpuMesh {
        GpuMesh {
            vertex_buffer: m.vertex_buffer,
            instance_buffer: m.instance_buffer,
            index_buffer: m.index_buffer,
            num_indices: m.num_indices,
            instance_id: self.internal.instances.take(),
        }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    pub fn create_model(&mut self, m: &CpuModel) -> GpuModel {
        GpuModel {
            mesh: self.create_mesh(&m.mesh),
            materials: m.materials.iter().map(|mat| self.create_material(mat)).collect(),
        }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    pub fn create_instanced_model(&mut self, m: &GpuModel) -> GpuModel {
        GpuModel {
            mesh: self.create_instanced_mesh(&m.mesh),
            materials: m.materials.clone(),
        }
    }

    #[must_use]
    fn create_depth_texture_int(
        runtime: &Runtime,
        database: &mut GpuObjectDatabase,
        settings: &Settings,
        label: Option<&str>,
    ) -> TextureId {
        TextureBuilder::new(runtime, database, settings)
            .with_label(label)
            .with_depth_texture()
            .submit()
    }

    #[must_use]
    fn create_texture_view_int(
        database: &mut GpuObjectDatabase,
        label: Option<&str>,
        texture: TextureId,
    ) -> TextureViewId {
        let texture = &database.textures[&texture];

        tracing::trace!("Creating texture view '{}'", label.unwrap_or("unnamed"));
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            ..Default::default()
        });
        database.insert_texture_view(view)
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

        let mut database = GpuObjectDatabase::default();

        let camera_buffer_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("camera-buffer-layout")
            .add_bind_group_layout_entry(
                0,
                ShaderStages::VERTEX,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<CameraUniform>() as _),
                },
            )
            .submit();

        let light_buffer_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("light-buffer-layout")
            .add_bind_group_layout_entry(
                0,
                ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<LightUniform>() as _),
                },
            )
            .submit();

        let material_buffer_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("material-buffer-layout")
            .add_bind_group_layout_entry(
                0,
                ShaderStages::FRAGMENT,
                BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
            )
            .add_bind_group_layout_entry(
                1,
                ShaderStages::FRAGMENT,
                BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            )
            .submit();

        let depth_texture = Self::create_depth_texture_int(&runtime, &mut database, settings, DEPTH_TEXTURE_LABEL);
        let depth_texture_view = Self::create_texture_view_int(&mut database, DEPTH_TEXTURE_VIEW_LABEL, depth_texture);

        Ok(Graphics {
            settings: settings.clone(),
            runtime,
            database,
            internal: InternalRuntimeData {
                camera_buffer_layout,
                light_buffer_layout,
                material_buffer_layout,
                depth_texture,
                depth_texture_view,
                instances: Urn::default(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::Reg;

    #[test]
    fn graphics_reg_macro() {
        type _RR = Reg![Graphics];
    }
}
