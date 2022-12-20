use std::{
    fs::File,
    path::Path,
    time::{Duration, Instant},
};

use ecs::{EventQueue, Reg, RegAdd, ResourceRegistry, SystemRegistry, LoopControl, ReceiverId};
use engine2::{
    events::winit_mappings::WindowEvent,
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics},
};
use file_manipulation::FilePathBuf;
use log::trace;
use winit::{
    event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, ModifiersState},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();

    let state: Orchestrator<Reg![], Reg![], Reg![], Reg![]> = Orchestrator::new().unwrap();

    event_loop.run(state.run(String::from("test"), false))
}

const DELTA_TIME: u64 = 50; // milliseconds
const MAX_FRAME_DURATION: u64 = 250; // milliseconds

type Resources<S> = RegAdd![AssetDatabase, Graphics, EventQueue<WindowEvent>, Statistics, S];

type World<S, F, D, R> = ecs::World<Resources<S>, F, D, R>;

pub struct Orchestrator<S, F, D, R> {
    world: World<S, F, D, R>,
    timers: Timers,
    window_event_receiver: ReceiverId<WindowEvent>,
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

        let mut world = World::try_default()?;
        let window_event_receiver = world.get_mut::<EventQueue<WindowEvent>>().subscribe::<WindowEvent>();

        Ok(Orchestrator {
            world,
            timers: Timers::default(),
            window_event_receiver,
        })
    }

    pub fn run(
        mut self,
        name: String,
        force_init: bool,
    ) -> impl 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) {
        move |event, event_loop, control_flow| {
            #[cfg(feature = "loopdbg")]
            let mut draw_bottom = false;
            #[cfg(feature = "loopdbg")]
            {
                match &event {
                    Event::NewEvents(_) => trace!("⬇"),
                    Event::RedrawEventsCleared | Event::LoopDestroyed => draw_bottom = true,
                    _ => (),
                }
                trace!("Event trace: {:?}", &event);
            }

            match event {
                Event::NewEvents(StartCause::Init) => self.init(event_loop, &name, force_init),
                Event::WindowEvent {
                    window_id,
                    event: window_event,
                } if self.world.borrow::<Graphics>().window_id().map_or(false, |wid| wid == window_id) => {
                    if let Ok(window_event) = window_event.try_into() {
                        self.input(window_event)
                    }
                }
                Event::MainEventsCleared => self.redraw(),
                Event::RedrawEventsCleared => self.maintain(control_flow),
                Event::LoopDestroyed => self.cleanup(),
                _ => (),
            }

            #[cfg(feature = "loopdbg")]
            if draw_bottom {
                trace!("⬆\n\n");
            }
        }
    }

    fn init<T>(&mut self, event_loop: &EventLoopWindowTarget<T>, name: &str, force_init: bool) {
        use pollster::FutureExt;

        self.world
            .get_mut::<AssetDatabase>()
            .initialize(name, force_init)
            .unwrap();
        self.world.get_mut::<Graphics>().initialize(event_loop).block_on();
    }

    /// Send the `winit` event to the internal event queue for further processing.
    fn input(&mut self, window_event: WindowEvent) {
        self.world.get_mut::<EventQueue<WindowEvent>>().send(window_event)
    }

    /// Update the game state (using [`World::update`](ecs::world::World::update) and
    /// [`World::fixed_update`](ecs::world::World::fixed_update)) and render the frame (using
    /// [`World::render`](ecs::world::World::render)).
    fn redraw(&mut self) {
        // Assess the duration of the last frame
        let frame_time = std::cmp::min(self.timers.loop_time.elapsed(), self.timers.max_frame_duration);
        self.timers.loop_time = Instant::now();
        self.timers.accumulator += frame_time;
        self.timers.dynamic_game_time += frame_time;

        // Call fixed update functions until the accumulated time buffer is empty
        while self.timers.accumulator >= self.timers.delta_time {
            self.world
                .fixed_update(&self.timers.fixed_game_time, &self.timers.delta_time);
            self.timers.accumulator -= self.timers.delta_time;
            self.timers.fixed_game_time += self.timers.delta_time;
        }

        // Call the dynamic update and render functions
        self.world.update(&self.timers.dynamic_game_time, &frame_time);
        self.world.render(&self.timers.dynamic_game_time, &frame_time);

        // Update the frame time statistics
        self.world.get_mut::<Statistics>().update_loop_times(frame_time);
    }

    /// Perform maintenance tasks necessary in each game loop iteration
    fn maintain(&mut self, control_flow: &mut ControlFlow) {
        // The standard action is to Poll
        *control_flow = ControlFlow::Poll;

        // Call the maintenance method of [`World`](ecs::World)
        if let LoopControl::Abort = self.world.maintain() {
            *control_flow = ControlFlow::Exit;
        }

        // Process window events
        let events = self.world.get_mut::<EventQueue<WindowEvent>>().receive(&self.window_event_receiver);
        for event in events {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Released, virtual_keycode: Some(VirtualKeyCode::Q), .. }, .. } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        }

        #[cfg(feature = "loopdbg")]
        trace!("Control flow: {:?}", control_flow);
    }

    fn cleanup(&mut self) {
        self.world.clear();
    }
}

#[derive(Debug)]
struct Timers {
    loop_time: Instant,
    accumulator: Duration,
    dynamic_game_time: Duration,
    fixed_game_time: Duration,
    delta_time: Duration,
    max_frame_duration: Duration,
}

impl Default for Timers {
    fn default() -> Self {
        Timers {
            loop_time: Instant::now(),
            accumulator: Duration::default(),
            dynamic_game_time: Duration::default(),
            fixed_game_time: Duration::default(),
            delta_time: Duration::from_millis(DELTA_TIME),
            max_frame_duration: Duration::from_millis(MAX_FRAME_DURATION),
        }
    }
}
