use wgpu::{DeviceDescriptor, RequestAdapterOptions, TextureUsages};
use winit::{event_loop::EventLoopWindowTarget, window::Fullscreen};

use crate::resources::graphics::settings::Settings;

#[derive(Debug)]
pub struct Runtime<'a> {
    pub window: std::sync::Arc<winit::window::Window>,
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub max_size: winit::dpi::PhysicalSize<u32>,
}

impl<'a> Runtime<'a> {
    pub async fn new<T>(event_loop: &EventLoopWindowTarget<T>, settings: &Settings) -> Runtime<'a> {
        let primary_monitor = event_loop.primary_monitor();
        let window = std::sync::Arc::new(
            winit::window::WindowBuilder::new()
                .with_fullscreen(Some(Fullscreen::Borderless(primary_monitor)))
                .build(event_loop)
                .unwrap(),
        );

        let size = window.inner_size();
        tracing::debug!("Physical window size: {:?}", &size);

        let max_size = window.current_monitor().unwrap().size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: settings.backends,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: settings.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        tracing::debug!("Supported adapter features: {:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: settings.required_features,
                    required_limits: settings.required_limits.clone(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        tracing::debug!("Supported texture formats: {:?}", &capabilities.formats);

        let texture_format = capabilities
            .formats
            .iter()
            .find(|&tf| tf == &settings.preferred_texture_format)
            .unwrap_or(&capabilities.formats[0]);
        tracing::debug!("Choosing texture format: {:?}", &texture_format);

        tracing::debug!("Supported present modes: {:?}", &capabilities.present_modes);

        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: *texture_format,
            width: size.width,
            height: size.height,
            present_mode: settings.present_mode,
            desired_maximum_frame_latency: 0,
            alpha_mode: settings.alpha_mode,
            view_formats: vec![*texture_format],
        };
        surface.configure(&device, &config);

        Runtime {
            window,
            surface,
            device,
            queue,
            config,
            size,
            max_size,
        }
    }
}
