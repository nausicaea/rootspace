use log::trace;
use ecs::{ResourceRegistry, SystemRegistry, Reg};
use winit::{event_loop::{EventLoop, EventLoopWindowTarget, ControlFlow}, window::{WindowBuilder, Window}, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, StartCause}};

fn main() {
    env_logger::init();


    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let state: State<Reg![], Reg![], Reg![], Reg![]> = State::new(window, "wgpu", false).unwrap();

    event_loop.run(state.event_handler_factory())
}

type World<S, F, D, R> = ecs::World<S, F, D, R>;

struct State<S, F, D, R> {
    world: World<S, F, D, R>,
    window: Window,
}

impl<S, F, D, R> State<S, F, D, R> 
where
    S: 'static + ResourceRegistry,
    F: 'static + SystemRegistry,
    D: 'static + SystemRegistry,
    R: 'static + SystemRegistry,
{
    fn new<N: AsRef<str>>(window: Window, name: N, force_init: bool) -> anyhow::Result<Self> {
        use try_default::TryDefault;

        let world = World::try_default()?;

        Ok(State { world, window })
    }

    async fn init(&mut self) {
        let size = self.window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&self.window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
    }

    fn event_handler_factory(mut self) -> impl 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) {
        move |event, _event_loop, control_flow| {
            use pollster::FutureExt;

            trace!("{:?}", &event);
            match event { 
                Event::NewEvents(StartCause::Init) => self.init().block_on(),
                Event::WindowEvent { window_id, event: ref e } if window_id == self.window.id() => {
                    match e {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => (),
                    }
                },
                _ => (),
            }
        }
    }
}
