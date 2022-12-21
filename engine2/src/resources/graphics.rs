use ecs::{Resource, SerializationName};
use log::debug;
use serde::{Deserialize, Serialize};
use wgpu::{DeviceDescriptor, Instance, RequestAdapterOptions, TextureUsages};
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Graphics {
    settings: Settings,
    #[serde(skip)]
    runtime: Option<Runtime>,
}

impl Graphics {
    pub async fn initialize<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        let window = WindowBuilder::new().build(event_loop).unwrap();

        let instance = Instance::new(self.settings.backends);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: self.settings.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        debug!("Supported adapter features: {:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: self.settings.features,
                    limits: self.settings.limits.clone(),
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
            .filter(|&tf| tf == &self.settings.preferred_texture_format)
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
            present_mode: self.settings.present_mode,
            alpha_mode: self.settings.alpha_mode,
        };
        surface.configure(&device, &config);

        self.runtime = Some(Runtime {
            window,
            surface,
            device,
            queue,
            config,
        });
    }

    pub fn window_id(&self) -> Option<WindowId> {
        self.runtime.as_ref().map(|rt| rt.window.id())
    }
}

impl Resource for Graphics {}

impl SerializationName for Graphics {}

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    backends: wgpu::Backends,
    power_preference: wgpu::PowerPreference,
    features: wgpu::Features,
    limits: wgpu::Limits,
    preferred_texture_format: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    alpha_mode: wgpu::CompositeAlphaMode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::LowPower,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            preferred_texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        }
    }
}

#[derive(Debug)]
struct Runtime {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}
