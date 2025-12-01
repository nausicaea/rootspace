use crate::base::settings::Settings;
use anyhow::{Context, anyhow};
use wgpu::{DeviceDescriptor, RequestAdapterOptions, TextureUsages};
use winit::{event_loop::EventLoopWindowTarget, window::Fullscreen};

#[derive(Debug)]
pub struct Runtime<'a> {
    pub window: std::sync::Arc<winit::window::Window>,
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'a>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub max_size: winit::dpi::PhysicalSize<u32>,
}

impl<'a> Runtime<'a> {
    pub async fn new<T>(event_loop: &EventLoopWindowTarget<T>, settings: &Settings) -> anyhow::Result<Runtime<'a>> {
        let primary_monitor = event_loop.primary_monitor();
        let window = std::sync::Arc::new(
            winit::window::WindowBuilder::new()
                .with_fullscreen(Some(Fullscreen::Borderless(primary_monitor)))
                .build(event_loop)
                .context("Creating a window")?,
        );

        let size = window.inner_size();
        tracing::debug!("Physical window size: {:?}", &size);

        let max_size = window
            .current_monitor()
            .ok_or_else(|| anyhow!("No monitor assigned to the current window"))?
            .size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: settings.backends,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: settings.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        tracing::debug!("Supported adapter features: {:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_features: settings.required_features,
                required_limits: settings.required_limits.clone(),
                label: None,
                memory_hints: Default::default(),
                trace: Default::default(),
                experimental_features: Default::default(),
            })
            .await?;

        let capabilities = surface.get_capabilities(&adapter);
        tracing::debug!("Supported texture formats: {:?}", &capabilities.formats);

        let texture_format = capabilities
            .formats
            .iter()
            .find(|&tf| tf == &settings.preferred_texture_format)
            .unwrap_or(&capabilities.formats[0]);
        tracing::debug!("Choosing surface texture format: {:?}", &texture_format);

        tracing::debug!("Supported present modes: {:?}", &capabilities.present_modes);

        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: *texture_format,
            width: size.width,
            height: size.height,
            present_mode: settings.present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: settings.alpha_mode,
            view_formats: vec![*texture_format],
        };
        surface.configure(&device, &config);

        Ok(Runtime {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            size,
            max_size,
        })
    }
}
