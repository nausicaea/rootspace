use std::{fs::File, path::Path, time::{Instant, Duration}};

use ecs::{Reg, RegAdd, ResourceRegistry, SystemRegistry, EventQueue};
use engine2::{resources::{asset_database::AssetDatabase, graphics::Graphics}, events::winit_mappings::WindowEvent};
use file_manipulation::FilePathBuf;
use log::trace;
use winit::{
    event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();

    let state: Orchestrator<Reg![], Reg![], Reg![], Reg![]> = Orchestrator::new().unwrap();

    event_loop.run(state.run(String::from("test"), false))
}

// FIXME: WindowEvent<'_> is not serializable
type Resources<S> = RegAdd![AssetDatabase, Graphics, EventQueue<WindowEvent>, S];

type World<S, F, D, R> = ecs::World<Resources<S>, F, D, R>;

pub struct Orchestrator<S, F, D, R> {
    world: World<S, F, D, R>,
}

impl<S, F, D, R> Orchestrator<S, F, D, R>
where
    S: 'static + ResourceRegistry,
    F: 'static + SystemRegistry,
    D: 'static + SystemRegistry,
    R: 'static + SystemRegistry,
{
    pub fn new() -> anyhow::Result<Self> {
        use try_default::TryDefault;

        let world = World::try_default()?;

        Ok(Orchestrator { world })
    }

    pub fn run(
        mut self, name: String, force_init: bool,
    ) -> impl 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) {
        move |event, event_loop, control_flow| {
            trace!("{:?}", &event);
            match event {
                Event::NewEvents(StartCause::Init) => self.init(event_loop, &name, force_init),
                Event::WindowEvent { event: window_event, .. } => if let Ok(window_event) = window_event.try_into() { self.input(window_event) },
                Event::MainEventsCleared => self.request_redraw(),
                Event::RedrawRequested(_) => self.redraw(),
                Event::RedrawEventsCleared => *control_flow = self.maintain(),
                _ => (),
            }
        }
    }

    fn init<T>(&mut self, event_loop: &EventLoopWindowTarget<T>, name: &str, force_init: bool) {
        use pollster::FutureExt;

        self.world.get_mut::<AssetDatabase>().initialize(name, force_init).unwrap();
        self.world.get_mut::<Graphics>().initialize(event_loop).block_on();
    }

    fn input(&mut self, window_event: WindowEvent) {
        self.world.get_mut::<EventQueue<WindowEvent>>().send(window_event)
    }

    fn request_redraw(&mut self) {
        self.world.get_mut::<Graphics>().request_redraw();
    }

    fn redraw(&mut self) {
    }

    fn maintain(&mut self) -> ControlFlow {
        todo!()
    }

}

#[derive(Debug)]
struct Timing {
    loop_time: Instant,
    accumulator: Duration,
    dynamic_game_time: Duration,
    fixed_game_time: Duration,
}
