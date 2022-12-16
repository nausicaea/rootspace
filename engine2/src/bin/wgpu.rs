use log::trace;
use ecs::{ResourceRegistry, SystemRegistry, Reg};
use winit::{event_loop::{EventLoop, EventLoopWindowTarget, ControlFlow}, window::{WindowBuilder, Window}, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, StartCause}};

fn main() {
    use pollster::FutureExt;

    env_logger::init();

    let state: State<Reg![], Reg![], Reg![], Reg![]> = State::new("wgpu", false).block_on().unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(state.event_handler_factory(window))
}

type World<S, F, D, R> = ecs::World<S, F, D, R>;

struct State<S, F, D, R> {
    world: World<S, F, D, R>,
}

impl<S, F, D, R> State<S, F, D, R> 
where
    S: ResourceRegistry,
    F: SystemRegistry,
    D: SystemRegistry,
    R: SystemRegistry,
{
    async fn new<N: AsRef<str>>(name: N, force_init: bool) -> anyhow::Result<Self> {
        use try_default::TryDefault;

        let world = World::try_default()?;

        Ok(State { world })
    }

    fn event_handler_factory(self, window: Window) -> impl 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) {
        move |event, _event_loop, control_flow| {
            trace!("{:?}", &event);
            match event { 
                Event::NewEvents(StartCause::Init) => (),
                Event::WindowEvent { window_id: w, event: ref e } if w == window.id() => {
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
