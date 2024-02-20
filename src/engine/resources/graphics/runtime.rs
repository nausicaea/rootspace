use log::debug;
use wgpu::{DeviceDescriptor, RequestAdapterOptions, TextureUsages};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Fullscreen;

#[derive(Debug)]
pub struct Runtime {
    pub window: winit::window::Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub max_size: winit::dpi::PhysicalSize<u32>,
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
        let window = winit::window::WindowBuilder::new()
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(event_loop).unwrap();

        let max_size = window.current_monitor().unwrap().size();

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
            max_size,
        }
    }
}
