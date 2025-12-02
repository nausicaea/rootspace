use std::mem::size_of;

use wgpu::{BindingType, BufferAddress, BufferBindingType, BufferSize, BufferUsages, ShaderStages};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;

use super::assets::cpu_material::CpuMaterial;
use super::assets::cpu_mesh::CpuMesh;
use super::assets::cpu_model::CpuModel;
use super::assets::cpu_texture::CpuTexture;
use crate::base::bind_group_builder::BindGroupBuilder;
use crate::base::bind_group_layout_builder::BindGroupLayoutBuilder;
use crate::base::camera_uniform::CameraUniform;
use crate::base::encoder::Encoder;
use crate::base::gpu_material::GpuMaterial;
use crate::base::gpu_mesh::GpuMesh;
use crate::base::gpu_model::GpuModel;
use crate::base::gpu_object_database::GpuObjectDatabase;
use crate::base::gpu_texture::GpuTexture;
use crate::base::ids::{BindGroupLayoutId, BufferId, ShaderModuleId, TextureId, TextureViewId};
use crate::base::instance::Instance;
use crate::base::internal_runtime_data::InternalRuntimeData;
use crate::base::light_uniform::LightUniform;
use crate::base::material_uniform::MaterialUniform;
use crate::base::render_pipeline_builder::RenderPipelineBuilder;
use crate::base::runtime::Runtime;
use crate::base::sampler_builder::SamplerBuilder;
use crate::base::settings::Settings;
use crate::base::texture_builder::TextureBuilder;
use ecs::{resource::Resource, with_dependencies::WithDependencies};
use urn::Urn;

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

    pub fn gen_instance_report(&self) -> Option<wgpu_core::global::GlobalReport> {
        self.runtime.instance.generate_report()
    }

    pub fn gen_surface_capabilities(&self) -> wgpu::SurfaceCapabilities {
        self.runtime.surface.get_capabilities(&self.runtime.adapter)
    }

    pub fn gen_adapter_features(&self) -> wgpu::Features {
        self.runtime.adapter.features()
    }

    pub fn gen_adapter_limits(&self) -> wgpu::Limits {
        self.runtime.adapter.limits()
    }

    pub fn gen_adapter_downlevel_capabilities(&self) -> wgpu::DownlevelCapabilities {
        self.runtime.adapter.get_downlevel_capabilities()
    }

    pub fn gen_adapter_info(&self) -> wgpu::AdapterInfo {
        self.runtime.adapter.get_info()
    }

    pub fn gen_device_allocator_report(&self) -> Option<wgpu::AllocatorReport> {
        self.runtime.device.generate_allocator_report()
    }

    pub fn limits(&self) -> wgpu::Limits {
        self.runtime.device.limits()
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.runtime.window.id()
    }

    pub fn window_inner_size(&self) -> PhysicalSize<u32> {
        self.runtime.window.inner_size()
    }

    pub fn request_redraw(&self) {
        self.runtime.window.request_redraw()
    }

    pub fn reconfigure(&mut self) {
        self.resize(self.runtime.size)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
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

    pub fn camera_bind_group_layout(&self) -> BindGroupLayoutId {
        self.internal.camera_bind_group_layout
    }

    pub fn light_bind_group_layout(&self) -> BindGroupLayoutId {
        self.internal.light_bind_group_layout
    }

    pub fn material_bind_group_layout(&self) -> BindGroupLayoutId {
        self.internal.material_bind_group_layout
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

    pub fn create_encoder(&self, label: Option<&str>) -> Result<Encoder<'_>, wgpu::SurfaceError> {
        Encoder::new(
            label,
            &self.runtime,
            &self.settings,
            &self.database,
            self.internal.depth_texture_view,
        )
    }

    #[must_use]
    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder<'_, '_, '_> {
        RenderPipelineBuilder::new(&self.runtime, &mut self.database, &self.settings)
    }

    #[must_use]
    pub fn create_bind_group_layout(&mut self) -> BindGroupLayoutBuilder<'_> {
        BindGroupLayoutBuilder::new(&self.runtime, &mut self.database)
    }

    #[must_use]
    pub fn create_bind_group(&mut self, layout: BindGroupLayoutId) -> BindGroupBuilder<'_> {
        BindGroupBuilder::new(&self.runtime, &mut self.database, layout)
    }

    #[must_use]
    pub fn create_buffer<A: Into<BufferAddress>>(
        &mut self,
        label: Option<&str>,
        size: A,
        usage: BufferUsages,
    ) -> BufferId {
        tracing::trace!("Creating buffer '{}'", label.unwrap_or("unnamed"));
        let buf = self.runtime.device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: size.into(),
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
    fn create_gpu_texture(&mut self, t: &CpuTexture) -> GpuTexture {
        let texture = TextureBuilder::new(&self.runtime, &mut self.database, &self.settings)
            .with_label(t.label.as_ref().map(|l| format!("{}:texture", &l)).as_deref())
            .with_image(&t.image)
            .submit();
        let view = self.create_texture_view(
            t.label.as_ref().map(|l| format!("{}:texture-view", &l)).as_deref(),
            texture,
        );
        let sampler = self
            .create_sampler()
            .with_label(t.label.as_ref().map(|l| format!("{}:texture-sampler", &l)).as_deref())
            .submit();

        GpuTexture { texture, view, sampler }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    fn create_gpu_material(&mut self, m: &CpuMaterial) -> GpuMaterial {
        let texture = self.create_gpu_texture(&m.texture);

        let material = self.create_buffer_init(
            Some("material-buffer"),
            BufferUsages::UNIFORM,
            &[MaterialUniform {
                ambient_reflectivity: m.ambient_reflectivity,
                diffuse_reflectivity: m.diffuse_reflectivity,
                specular_reflectivity: m.specular_reflectivity,
                smoothness: m.smoothness,
            }],
        );

        let layout = self.material_bind_group_layout();
        let bind_group = self
            .create_bind_group(layout)
            .with_label(m.label.as_ref().map(|l| format!("{}:bind-group", &l)).as_deref())
            .add_texture_view(0, texture.view)
            .add_sampler(1, texture.sampler)
            .add_entire_buffer(2, material)
            .submit();

        GpuMaterial { texture, bind_group }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    fn create_gpu_mesh(&mut self, m: &CpuMesh) -> GpuMesh {
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
    fn create_instanced_gpu_mesh(&mut self, m: &GpuMesh) -> GpuMesh {
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
    pub fn create_gpu_model(&mut self, m: &CpuModel) -> GpuModel {
        GpuModel {
            mesh: self.create_gpu_mesh(&m.mesh),
            materials: m.materials.iter().map(|mat| self.create_gpu_material(mat)).collect(),
        }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    pub fn create_instanced_gpu_model(&mut self, m: &GpuModel) -> GpuModel {
        GpuModel {
            mesh: self.create_instanced_gpu_mesh(&m.mesh),
            materials: m.materials.clone(),
        }
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    pub fn create_texture(&mut self) -> TextureBuilder<'_> {
        TextureBuilder::new(&self.runtime, &mut self.database, &self.settings)
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    pub fn create_texture_view(&mut self, label: Option<&str>, texture: TextureId) -> TextureViewId {
        Self::create_texture_view_int(&mut self.database, label, texture)
    }

    #[tracing::instrument(skip_all)]
    #[must_use]
    pub fn create_sampler(&mut self) -> SamplerBuilder<'_> {
        SamplerBuilder::new(&self.runtime, &mut self.database)
    }

    /// This function does not bind `self` on purpose because it needs to work during the constructor.
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
    async fn with_deps(deps: &D) -> anyhow::Result<Self> {
        let settings = deps.settings();
        let runtime = Runtime::new(deps.event_loop(), settings).await?;

        let mut database = GpuObjectDatabase::default();

        let camera_bind_group_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("camera-bind-group-layout")
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

        let light_bind_group_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("light-bind-group-layout")
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

        let material_bind_group_layout = BindGroupLayoutBuilder::new(&runtime, &mut database)
            .with_label("material-bind-group-layout-layout")
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
            .add_bind_group_layout_entry(
                2,
                ShaderStages::FRAGMENT,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<MaterialUniform>() as _),
                },
            )
            .submit();

        let depth_texture = Self::create_depth_texture_int(&runtime, &mut database, settings, DEPTH_TEXTURE_LABEL);
        let depth_texture_view = Self::create_texture_view_int(&mut database, DEPTH_TEXTURE_VIEW_LABEL, depth_texture);

        Ok(Graphics {
            settings: settings.clone(),
            runtime,
            database,
            internal: InternalRuntimeData {
                camera_bind_group_layout,
                light_bind_group_layout,
                material_bind_group_layout,
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
