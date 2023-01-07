use std::time::{Duration, Instant};

use ecs::{
    with_dependencies::WithDependencies, EventQueue, LoopControl, ReceiverId, ResourceRegistry, SystemRegistry,
    WithResources, WorldEvent,
};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoopWindowTarget},
};

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    registry::{RRegistry, RSRegistry, USRegistry},
    resources::{
        asset_database::AssetDatabaseDeps,
        graphics::{Graphics, GraphicsDeps},
        statistics::Statistics,
    },
};

const DELTA_TIME: u64 = 50; // milliseconds
const MAX_FRAME_DURATION: u64 = 250; // milliseconds

pub struct Orchestrator {
    world: ecs::World,
    timers: Timers,
    window_event_receiver: ReceiverId<WindowEvent>,
    engine_event_receiver: ReceiverId<EngineEvent>,
}

impl Orchestrator {
    pub fn with_dependencies<RR, FUSR, USR, RSR, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        RR: ResourceRegistry + WithDependencies<D>,
        FUSR: SystemRegistry + WithResources,
        USR: SystemRegistry + WithResources,
        RSR: SystemRegistry + WithResources,
        D: GraphicsDeps + AssetDatabaseDeps,
    {
        let mut world =
            ecs::World::with_dependencies::<RRegistry<RR>, FUSR, USRegistry<USR>, RSRegistry<RSR>, _>(deps)?;
        let window_event_receiver = world.get_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        let engine_event_receiver = world.get_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        Ok(Orchestrator {
            world,
            timers: Timers::default(),
            window_event_receiver,
            engine_event_receiver,
        })
    }

    pub fn run(mut self) -> impl 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) {
        move |event, _event_loop, control_flow| {
            #[cfg(feature = "loopdbg")]
            let mut draw_bottom = false;
            #[cfg(feature = "loopdbg")]
            {
                match &event {
                    Event::NewEvents(_) => log::trace!("⬇"),
                    Event::RedrawEventsCleared | Event::LoopDestroyed => draw_bottom = true,
                    _ => (),
                }
                log::trace!("Event trace: {:?}", &event);
            }

            match event {
                Event::WindowEvent {
                    window_id,
                    event: window_event,
                } if self.world.borrow::<Graphics>().window_id() == window_id => {
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
                log::trace!("⬆\n\n");
            }
        }
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
        // self.world.borrow::<Graphics>().render().unwrap();

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
        let events = self
            .world
            .get_mut::<EventQueue<WindowEvent>>()
            .receive(&self.window_event_receiver);
        for event in events {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        }

        // Process engine events
        let events = self
            .world
            .get_mut::<EventQueue<EngineEvent>>()
            .receive(&self.engine_event_receiver);
        for event in events {
            match event {
                EngineEvent::AboutToAbort => self.world.get_mut::<EventQueue<WorldEvent>>().send(WorldEvent::Abort),
                _ => (),
            }
        }

        #[cfg(feature = "loopdbg")]
        log::trace!("Control flow: {:?}", control_flow);
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
