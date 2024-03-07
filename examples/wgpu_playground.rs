#![allow(dead_code)]

use log::debug;
use std::sync::Arc;
use wgpu::{
    Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode,
    Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages,
};
use winit::{
    event::{Event, StartCause},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

#[tokio::main]
async fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let state = State::new(window).await;

    event_loop.run(run(state)).unwrap();
}

fn run<T: 'static>(mut state: State<'static>) -> impl 'static + FnMut(Event<T>, &EventLoopWindowTarget<T>) {
    move |event, event_loop| if let Event::NewEvents(StartCause::Init) = event {
        state.init();
        event_loop.exit()
    }
}

#[derive(Debug)]
struct State<'a> {
    window: Arc<Window>,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,
}

impl<'a> State<'a> {
    async fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        debug!("Supported adapter features: {:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    label: Some("No features, default limits"),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let size = window.inner_size();
        debug!("Physical window size: {:?}", &size);

        let capabilities = surface.get_capabilities(&adapter);
        let texture_formats = capabilities.formats;
        debug!("Supported texture formats: {:?}", &texture_formats);

        let texture_format = texture_formats[0];
        debug!("Choosing texture format: {:?}", &texture_format);

        let present_modes = capabilities.present_modes;
        debug!("Supported present modes: {:?}", &present_modes);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![texture_format],
        };
        surface.configure(&device, &surface_config);

        State {
            window,
            surface,
            device,
            queue,
            surface_config,
        }
    }

    fn init(&mut self) {}
}
