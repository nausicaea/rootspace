use std::time::{Duration, Instant};

use winit::{event::Event, event_loop::EventLoopWindowTarget};

use crate::ecs::entity::Entity;
use crate::ecs::event_queue::receiver_id::ReceiverId;
use crate::ecs::event_queue::EventQueue;
use crate::ecs::loop_control::LoopControl;
use crate::ecs::registry::{ResourceRegistry, SystemRegistry};
use crate::ecs::storage::Storage;
use crate::ecs::with_dependencies::WithDependencies;
use crate::ecs::with_resources::WithResources;
use crate::ecs::world::event::WorldEvent;
use crate::ecs::world::World;
use crate::engine::assets::scene::Scene;
use crate::engine::components::camera::Camera;
use crate::engine::components::info::Info;
use crate::engine::components::renderable::Renderable;
use crate::engine::components::status::Status;
use crate::engine::components::transform::Transform;
use crate::engine::components::ui_transform::UiTransform;
use crate::engine::events::engine_event::EngineEvent;
use crate::engine::registry::{RRegistry, RSRegistry, USRegistry};
use crate::engine::resources::asset_database::{AssetDatabase, AssetDatabaseDeps};
use crate::engine::resources::graphics::{Graphics, GraphicsDeps};
use crate::engine::resources::statistics::Statistics;
use winit::event::WindowEvent;
use crate::trace_loop;

const DELTA_TIME: u64 = 50; // milliseconds
const MAX_FRAME_DURATION: u64 = 250; // milliseconds

pub struct Orchestrator {
    world: World,
    timers: Timers,
    world_event_receiver: ReceiverId<WorldEvent>,
    engine_event_receiver: ReceiverId<EngineEvent>,
}

impl Orchestrator {
    pub fn with_dependencies<RR, FUSR, USR, RSR, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        RR: ResourceRegistry + WithDependencies<D>,
        FUSR: SystemRegistry + WithResources,
        USR: SystemRegistry + WithResources,
        RSR: SystemRegistry + WithResources,
        D: GraphicsDeps + AssetDatabaseDeps + OrchestratorDeps,
    {
        let mut world = World::with_dependencies::<RRegistry<RR>, FUSR, USRegistry<USR>, RSRegistry<RSR>, _>(deps)?;
        let world_event_receiver = world.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>();
        let engine_event_receiver = world.get_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        world
            .borrow::<AssetDatabase>()
            .load_asset::<Scene, _>(world.resources(), "scenes", deps.main_scene())?;

        Ok(Orchestrator {
            world,
            timers: Timers::default(),
            world_event_receiver,
            engine_event_receiver,
        })
    }

    pub fn run(mut self) -> impl 'static + FnMut(Event<()>, &EventLoopWindowTarget<()>) {
        move |event, elwt| {
            #[cfg(feature = "dbg-loop")]
            let mut draw_bottom = false;
            #[cfg(feature = "dbg-loop")]
            {
                match &event {
                    Event::NewEvents(_) => {
                        log::trace!("⬇");
                    }
                    Event::AboutToWait | Event::LoopExiting => {
                        draw_bottom = true;
                    }
                    _ => (),
                }
                log::trace!("Event trace: {:?}", &event);
            }

            let main_window_id = self.world.borrow::<Graphics>().window_id();
            match event {
                Event::WindowEvent {
                    window_id,
                    event: window_event,
                } if main_window_id == window_id => match window_event {
                    WindowEvent::RedrawRequested => self.redraw(),
                    e => self.world.get_mut::<EventQueue<WindowEvent>>().send(e),
                },
                Event::AboutToWait => self.maintain(elwt),
                Event::LoopExiting => self.cleanup(),
                _ => (),
            }

            #[cfg(feature = "dbg-loop")]
            if draw_bottom {
                log::trace!("⬆\n\n");
            }
        }
    }

    /// Update the game state (using [`World::update`](ecs::world::World::update) and
    /// [`World::fixed_update`](ecs::world::World::fixed_update)) and render the frame (using
    /// [`World::render`](ecs::world::World::render)).
    fn redraw(&mut self) {
        trace_loop!("Redraw executing");
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
    fn maintain(&mut self, event_loop_window_target: &EventLoopWindowTarget<()>) {
        trace_loop!("Running maintenance");
        // Call the maintenance method of [`World`](ecs::World)
        if let LoopControl::Abort = self.world.maintain() {
            event_loop_window_target.exit();
        }

        // Process world events
        let events = self
            .world
            .get_mut::<EventQueue<WorldEvent>>()
            .receive(&self.world_event_receiver);
        for event in events {
            match event {
                WorldEvent::EntityDestroyed(e) => self.on_entity_destroyed(e),
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
                EngineEvent::AbortRequested => self.on_abort_requested(),
                _ => (),
            }
        }

        self.world.borrow::<Graphics>().request_redraw();
    }

    fn on_entity_destroyed(&mut self, entity: Entity) {
        log::trace!("Removing entity from components Status, Info, Transform, UiTransform, Renderable");
        self.world.get_components_mut::<Camera>().remove(entity);
        self.world.get_components_mut::<Status>().remove(entity);
        self.world.get_components_mut::<Info>().remove(entity);
        self.world.get_components_mut::<Transform>().remove(entity);
        self.world.get_components_mut::<UiTransform>().remove(entity);
        self.world.get_components_mut::<Renderable>().remove(entity);
    }

    fn on_abort_requested(&mut self) {
        self.world.get_mut::<EventQueue<WorldEvent>>().send(WorldEvent::Abort)
    }

    fn cleanup(&mut self) {
        self.world.clear();
    }
}

pub trait OrchestratorDeps {
    /// Specifies the name of the main scene
    fn main_scene(&self) -> &str;

    /// Specifies the name of the asset group scenes are stored in
    fn scene_group(&self) -> &str {
        "scenes"
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
