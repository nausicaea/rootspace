use log::debug;
use wgpu::{DeviceDescriptor, RequestAdapterOptions, TextureUsages};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Fullscreen;

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
    pub async fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        backends: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        required_features: wgpu::Features,
        required_limits: wgpu::Limits,
        preferred_texture_format: &wgpu::TextureFormat,
        present_mode: wgpu::PresentMode,
        alpha_mode: wgpu::CompositeAlphaMode,
    ) -> Runtime<'a> {
        let primary_monitor = event_loop.primary_monitor();
        let window = std::sync::Arc::new(
            winit::window::WindowBuilder::new()
                .with_fullscreen(Some(Fullscreen::Borderless(primary_monitor)))
                .build(event_loop)
                .unwrap(),
        );

        let size = window.inner_size();
        debug!("Physical window size: {:?}", &size);

        let max_size = window.current_monitor().unwrap().size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

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
                    required_features,
                    required_limits,
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let capabilities = surface.get_capabilities(&adapter);
        debug!("Supported texture formats: {:?}", &capabilities.formats);

        let texture_format = capabilities
            .formats
            .iter().find(|&tf| tf == preferred_texture_format)
            .unwrap_or(&capabilities.formats[0]);
        debug!("Choosing texture format: {:?}", &texture_format);

        debug!("Supported present modes: {:?}", &capabilities.present_modes);

        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: *texture_format,
            width: size.width,
            height: size.height,
            present_mode,
            desired_maximum_frame_latency: 0,
            alpha_mode,
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
