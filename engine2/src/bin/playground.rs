use winit::{event_loop::{EventLoop, EventLoopWindowTarget, ControlFlow}, window::{WindowBuilder, Window}, event::{Event, StartCause, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}};
use wgpu::{Instance, Backends, RequestAdapterOptions, PowerPreference, Limits, Features, DeviceDescriptor, SurfaceConfiguration, TextureUsages, PresentMode, CompositeAlphaMode, Surface, Device, Queue, QueueWriteBufferView};
use log::debug;

fn main() {
    use pollster::FutureExt;

    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let state = State::new(window).block_on();
    
    event_loop.run(run(state))
}

fn run<T>(mut state: State) -> impl 'static + FnMut(Event<T>, &EventLoopWindowTarget<T>, &mut ControlFlow) {
    move |event, _event_loop, control_flow| {

        match event {
            Event::NewEvents(StartCause::Init) => {
                state.init();
                *control_flow = ControlFlow::Exit;
            },
            _ => (),
        }
    }
}

#[derive(Debug)]
struct State {
    window: Window,
    surface: Surface,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,
}

impl State {
    async fn new(window: Window) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
        .await
            .unwrap();
        debug!("Supported adapter features: {:?}", adapter.features());

        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: Limits::default(),
                label: Some("No features, default limits"),
            },
            None, // Trace path
        ).await.unwrap();

        let size = window.inner_size();
        debug!("Physical window size: {:?}", &size);

        let texture_formats = surface.get_supported_formats(&adapter);
        debug!("Supported texture formats: {:?}", &texture_formats);

        let texture_format = texture_formats[0];
        debug!("Choosing texture format: {:?}", &texture_format);

        let present_modes = surface.get_supported_present_modes(&adapter);
        debug!("Supported present modes: {:?}", &present_modes);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Auto,
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

    fn init(&mut self) {
    }
}

