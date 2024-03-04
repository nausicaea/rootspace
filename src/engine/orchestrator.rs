use log::{info, trace};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

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
use crate::engine::registry::{RRegistry, USRegistry};
use crate::engine::resources::asset_database::{AssetDatabase, AssetDatabaseDeps};
use crate::engine::resources::graphics::{Graphics, GraphicsDeps};
use crate::engine::resources::statistics::Statistics;

use crate::engine::resources::rpc_settings::RpcDeps;
use crate::engine::systems::renderer::Renderer;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;

const STATS_DISPLAY_INTERVAL: Duration = Duration::from_secs(15);
const DELTA_TIME: Duration = Duration::from_millis(50);
const MAX_LOOP_DURATION: Duration = Duration::from_millis(250);
const MIN_LOOP_DURATION: Duration = Duration::from_millis(32);

pub struct Orchestrator {
    world: World,
    timers: Timers,
    world_event_receiver: ReceiverId<WorldEvent>,
    engine_event_receiver: ReceiverId<EngineEvent>,
    runtime: Arc<Runtime>,
}

impl Orchestrator {
    pub async fn with_dependencies<RR, FUSR, USR, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        RR: ResourceRegistry + WithDependencies<D>,
        FUSR: SystemRegistry + WithResources,
        USR: SystemRegistry + WithResources,
        D: GraphicsDeps + AssetDatabaseDeps + OrchestratorDeps + RpcDeps,
    {
        deps.event_loop().set_control_flow(ControlFlow::Poll);

        let mut world = World::with_dependencies::<RRegistry<RR>, FUSR, USRegistry<USR>, Renderer, _>(deps).await?;
        let world_event_receiver = world.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>();
        let engine_event_receiver = world.get_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        world
            .read::<AssetDatabase>()
            .load_asset::<Scene, _>(world.resources(), "scenes", deps.main_scene())
            .await?;

        Ok(Orchestrator {
            world,
            timers: Timers {
                delta_time: deps.delta_time(),
                max_loop_duration: deps.max_loop_duration(),
                min_loop_duration: deps.min_loop_duration(),
                #[cfg(feature = "dbg-loop")]
                stats_display_interval: deps.stats_display_interval(),
                ..Default::default()
            },
            world_event_receiver,
            engine_event_receiver,
            runtime: deps.runtime(),
        })
    }

    pub fn start(mut self) -> impl 'static + FnMut(Event<()>, &EventLoopWindowTarget<()>) {
        let rt = self.runtime.clone();

        move |event, elwt| {
            rt.block_on(self.run(event, elwt));
        }
    }

    async fn run(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        #[cfg(feature = "dbg-loop")]
        let mut draw_bottom = false;
        #[cfg(feature = "dbg-loop")]
        {
            match &event {
                Event::NewEvents(_) => {
                    trace!("⬇");
                }
                Event::AboutToWait | Event::LoopExiting => {
                    draw_bottom = true;
                }
                _ => (),
            }
            trace!("Event trace: {:?}", &event);
        }

        let main_window_id = self.world.read::<Graphics>().window_id();
        match event {
            Event::WindowEvent {
                window_id,
                event: window_event,
            } if main_window_id == window_id => match window_event {
                WindowEvent::RedrawRequested => self.redraw().await,
                e => self.world.get_mut::<EventQueue<WindowEvent>>().send(e),
            },
            Event::AboutToWait => self.maintain(elwt),
            Event::LoopExiting => self.on_exiting(),
            _ => (),
        }

        #[cfg(feature = "dbg-loop")]
        if draw_bottom {
            trace!("⬆\n\n");
        }
    }

    /// Update the game state (using [`World::update`](ecs::world::World::update) and
    /// [`World::fixed_update`](ecs::world::World::fixed_update)) and render the frame (using
    /// [`World::render`](ecs::world::World::render)).
    async fn redraw(&mut self) {
        // Assess the duration of the last frame
        let loop_time = std::cmp::min(self.timers.last_loop.elapsed(), self.timers.max_loop_duration);
        self.timers.last_loop = Instant::now();
        self.timers.accumulator += loop_time;
        self.timers.dynamic_game_time += loop_time;

        // Call fixed update functions until the accumulated time buffer is empty
        while self.timers.accumulator >= self.timers.delta_time {
            self.world
                .fixed_update(self.timers.fixed_game_time, self.timers.delta_time)
                .await;
            self.timers.accumulator -= self.timers.delta_time;
            self.timers.fixed_game_time += self.timers.delta_time;
        }

        // Call the dynamic update and render functions
        self.world.update(self.timers.dynamic_game_time, loop_time).await;
        self.world.render(self.timers.dynamic_game_time, loop_time).await;

        // Update the frame time statistics
        self.world.get_mut::<Statistics>().update_redraw_intervals(loop_time);
    }

    /// Perform maintenance tasks necessary in each game loop iteration
    fn maintain(&mut self, event_loop_window_target: &EventLoopWindowTarget<()>) {
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
                EngineEvent::Exit => self.on_exit(),
                _ => (),
            }
        }

        #[cfg(feature = "dbg-loop")]
        if self.timers.last_stats_display.elapsed() >= self.timers.stats_display_interval {
            self.timers.last_stats_display = Instant::now();
            info!("{}", self.world.read::<Statistics>());
        }

        if self.timers.last_loop.elapsed() >= self.timers.min_loop_duration {
            self.world.read::<Graphics>().request_redraw();
        }
    }

    fn on_entity_destroyed(&mut self, entity: Entity) {
        trace!("Removing entity from components Status, Info, Transform, UiTransform, Renderable");
        self.world.get_components_mut::<Camera>().remove(entity);
        self.world.get_components_mut::<Status>().remove(entity);
        self.world.get_components_mut::<Info>().remove(entity);
        self.world.get_components_mut::<Transform>().remove(entity);
        self.world.get_components_mut::<UiTransform>().remove(entity);
        self.world.get_components_mut::<Renderable>().remove(entity);
    }

    fn on_exit(&mut self) {
        info!("Exit requested");
        self.world.get_mut::<EventQueue<WorldEvent>>().send(WorldEvent::Exiting);
        self.world.read::<Graphics>().request_redraw();
    }

    fn on_exiting(&mut self) {
        info!("Exiting");
        self.world.clear();
    }
}

pub trait OrchestratorDeps {
    fn runtime(&self) -> Arc<Runtime>;

    /// Specifies the name of the main scene
    fn main_scene(&self) -> &str;

    /// Specifies the name of the asset group scenes are stored in
    fn scene_group(&self) -> &str {
        "scenes"
    }

    /// Specifies the upper bound for the duration of a loop iteration
    fn max_loop_duration(&self) -> Duration {
        MAX_LOOP_DURATION
    }

    /// Specifies the lower bound for the duration of a loop iteration
    fn min_loop_duration(&self) -> Duration {
        MIN_LOOP_DURATION
    }

    /// Specifies the fixed time interval
    fn delta_time(&self) -> Duration {
        DELTA_TIME
    }

    /// Specifies the interval at which stats information is shown (only applies with feature 'dbg-loop')
    fn stats_display_interval(&self) -> Duration {
        STATS_DISPLAY_INTERVAL
    }
}

#[derive(Debug)]
struct Timers {
    last_loop: Instant,
    accumulator: Duration,
    dynamic_game_time: Duration,
    fixed_game_time: Duration,
    delta_time: Duration,
    max_loop_duration: Duration,
    min_loop_duration: Duration,
    #[cfg(feature = "dbg-loop")]
    last_stats_display: Instant,
    #[cfg(feature = "dbg-loop")]
    stats_display_interval: Duration,
}

impl Default for Timers {
    fn default() -> Self {
        Timers {
            last_loop: Instant::now(),
            accumulator: Duration::default(),
            dynamic_game_time: Duration::default(),
            fixed_game_time: Duration::default(),
            delta_time: DELTA_TIME,
            max_loop_duration: MAX_LOOP_DURATION,
            min_loop_duration: MIN_LOOP_DURATION,
            #[cfg(feature = "dbg-loop")]
            last_stats_display: Instant::now(),
            #[cfg(feature = "dbg-loop")]
            stats_display_interval: STATS_DISPLAY_INTERVAL,
        }
    }
}
