use std::collections::HashMap;

use log::debug;
use wgpu::{DeviceDescriptor, RequestAdapterOptions, TextureUsages};
use winit::event_loop::EventLoopWindowTarget;

use super::{
    ids::{BindGroupId, BindGroupLayoutId, BufferId, PipelineId, SamplerId, TextureId},
    render_pass_builder::RenderPassBuilder,
    render_pipeline_builder::RenderPipelineBuilder,
    urn::Urn, settings::Settings,
};

#[derive(Debug)]
pub struct Runtime {
    pub window: winit::window::Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub indexes: Indexes,
    pub tables: Tables,
}

impl Runtime {
    pub async fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        backends: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        features: wgpu::Features,
        limits: wgpu::Limits,
        preferred_texture_format: &wgpu::TextureFormat,
        present_mode: wgpu::PresentMode,
        alpha_mode: wgpu::CompositeAlphaMode,
    ) -> Runtime {
        let window = winit::window::WindowBuilder::new().build(event_loop).unwrap();

        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        debug!("Supported adapter features: {:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features,
                    limits,
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let size = window.inner_size();
        debug!("Physical window size: {:?}", &size);

        let texture_formats = surface.get_supported_formats(&adapter);
        debug!("Supported texture formats: {:?}", &texture_formats);

        let texture_format = *texture_formats
            .iter()
            .filter(|&tf| tf == preferred_texture_format)
            .next()
            .unwrap_or(&texture_formats[0]);
        debug!("Choosing texture format: {:?}", &texture_format);

        let present_modes = surface.get_supported_present_modes(&adapter);
        debug!("Supported present modes: {:?}", &present_modes);

        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
        };
        surface.configure(&device, &config);

        Runtime {
            window,
            surface,
            device,
            queue,
            config,
            size,
            indexes: Indexes::default(),
            tables: Tables::default(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn create_render_pass<'rt>(&'rt self, settings: &'rt Settings) -> RenderPassBuilder {
        RenderPassBuilder::new(self, settings)
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        RenderPipelineBuilder::new(self)
    }

    pub fn create_bind_group_layout(
        &mut self,
        label: Option<&str>,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) -> BindGroupLayoutId {
        let bgl = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label, entries });

        let id = self.indexes.bind_group_layouts.take();
        self.tables.bind_group_layouts.insert(id, bgl);
        id
    }

    //pub fn create_bind_group(&mut self)
    //pub fn create_buffer(&mut self)
    //pub fn create_texture(&mut self)
}

#[derive(Debug, Default)]
pub struct Indexes {
    pub bind_group_layouts: Urn<BindGroupLayoutId>,
    pub bind_groups: Urn<BindGroupId>,
    pub buffers: Urn<BufferId>,
    pub textures: Urn<TextureId>,
    pub samplers: Urn<SamplerId>,
    pub render_pipelines: Urn<PipelineId>,
}

#[derive(Debug, Default)]
pub struct Tables {
    pub bind_group_layouts: HashMap<BindGroupLayoutId, wgpu::BindGroupLayout>,
    pub bind_groups: HashMap<BindGroupId, wgpu::BindGroup>,
    pub buffers: HashMap<BufferId, wgpu::Buffer>,
    pub textures: HashMap<TextureId, (wgpu::Texture, wgpu::TextureView)>,
    pub samplers: HashMap<SamplerId, wgpu::Sampler>,
    pub render_pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
}
