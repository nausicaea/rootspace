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

use super::registry::{FUSRegistry, MSRegistry};

const STATS_DISPLAY_INTERVAL: Duration = Duration::from_secs(15);
const DELTA_TIME: Duration = Duration::from_millis(50);
#[cfg(feature = "editor")]
const MAX_LOOP_DURATION: Duration = Duration::from_secs(2);
#[cfg(not(feature = "editor"))]
const MAX_LOOP_DURATION: Duration = Duration::from_millis(250);
const MIN_LOOP_DURATION: Option<Duration> = Some(Duration::from_millis(16));

pub struct Orchestrator {
    world: World,
    timers: Timers,
    #[cfg(feature = "editor")]
    window_event_receiver: ReceiverId<WindowEvent>,
    world_event_receiver: ReceiverId<WorldEvent>,
    engine_event_receiver: ReceiverId<EngineEvent>,
    runtime: Arc<Runtime>,
}

impl Orchestrator {
    pub async fn with_dependencies<RR, FUSR, USR, MSR, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        RR: ResourceRegistry + WithDependencies<D>,
        FUSR: SystemRegistry + WithResources,
        USR: SystemRegistry + WithResources,
        MSR: SystemRegistry + WithResources,
        D: GraphicsDeps + AssetDatabaseDeps + OrchestratorDeps + RpcDeps,
    {
        let mut world = World::with_dependencies::<
            RRegistry<RR>,
            FUSRegistry<FUSR>,
            USRegistry<USR>,
            Renderer,
            MSRegistry<MSR>,
            _,
        >(deps)
        .await?;

        #[cfg(feature = "editor")]
        let window_event_receiver = world.get_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        let world_event_receiver = world.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>();
        let engine_event_receiver = world.get_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        #[cfg(feature = "editor")]
        {
            let mut editor_scene = Scene::default();
            editor_scene
                .create_entity()
                .with_info(Info::builder().with_name("scene-gizmo").build())
                .submit();
            editor_scene.submit(world.resources(), "builtin", "editor").await?;
        }

        if let Some(main_scene) = deps.main_scene() {
            world
                .read::<AssetDatabase>()
                .load_asset::<Scene, _>(world.resources(), deps.scene_group(), main_scene)
                .await?;
        }

        Ok(Orchestrator {
            world,
            timers: Timers {
                delta_time: deps.delta_time(),
                max_loop_duration: deps.max_loop_duration(),
                #[cfg(not(feature = "editor"))]
                min_loop_duration: deps.min_loop_duration(),
                #[cfg(feature = "dbg-loop")]
                stats_display_interval: deps.stats_display_interval(),
                ..Default::default()
            },
            #[cfg(feature = "editor")]
            window_event_receiver,
            world_event_receiver,
            engine_event_receiver,
            runtime: deps.runtime(),
        })
    }

    /// Creates and returns a closure that is run by
    /// [`EventLoop::run`](winit::event_loop::EventLoop::run) every time `winit` received an event
    /// from the operating system. Internally, the closure instructs the asynchronous runtime to
    /// block on [`Orchestrator::run`](crate::engine::orchestrator::Orchestrator::run), which does
    /// the actual work.
    pub fn start(mut self) -> impl 'static + FnMut(Event<()>, &EventLoopWindowTarget<()>) {
        let rt = self.runtime.clone();

        move |event, elwt| {
            rt.block_on(self.run(event, elwt));
        }
    }

    /// Handles an event from `winit` and the operating system, by:
    /// 1. Initiating window redrawing with
    ///    [`Orchestrator::redraw`](crate::engine::orchestrator::Orchestrator::redraw)
    /// 2. Running regular maintenance with
    ///    [`Orchestrator::maintain`](crate::engine::orchestrator::Orchestrator::maintain)
    /// 3. Shutting down cleanly at the end of the engine lifecycle with
    ///    [`Orchestrator::on_exiting`](crate::engine::orchestrator::Orchestrator::on_exiting)
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
            Event::AboutToWait => self.maintain(elwt).await,
            Event::LoopExiting => self.on_exiting(),
            _ => (),
        }

        #[cfg(feature = "dbg-loop")]
        if draw_bottom {
            trace!("⬆\n\n");
        }
    }

    /// Update and render the engine state:
    /// 1. Call [`World::fixed_update`](crate::ecs::world::World::fixed_update) with fixed
    ///    time intervals by guaranteeing that missed updates are caught up with.
    /// 2. Call [`World::update`](crate::ecs::world::World::update) once per redraw event.
    /// 3. Call [`World::render`](crate::ecs::world::World::render) once per redraw event.
    /// 4. Update performance statistics in [`Statistics`](crate::engine::resources::statistics::Statistics)
    async fn redraw(&mut self) {
        // Assess the duration of the last frame
        let loop_time = std::cmp::min(self.timers.last_redraw.elapsed(), self.timers.max_loop_duration);
        self.timers.last_redraw = Instant::now();
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

    /// Perform maintenance tasks necessary in each game loop iteration. Also schedules redraw
    /// events based on loop timing information.
    /// Calls [`World::maintain`](crate::ecs::world::World::maintain`) after all other events
    /// have been handled.
    async fn maintain(&mut self, event_loop_window_target: &EventLoopWindowTarget<()>) {
        // Update maintenance statistics
        self.world
            .get_mut::<Statistics>()
            .update_maintenance_intervals(self.timers.last_maintenance.elapsed());
        self.timers.last_maintenance = Instant::now();

        // Process window events
        #[cfg(feature = "editor")]
        let mut window_interaction_received = false;
        #[cfg(feature = "editor")]
        {
            let events = self
                .world
                .get_mut::<EventQueue<WindowEvent>>()
                .receive(&self.window_event_receiver);
            for event in events {
                use WindowEvent::*;
                match event {
                    Resized(_)
                    | Moved(_)
                    | DroppedFile(_)
                    | Focused(_)
                    | KeyboardInput { .. }
                    | MouseWheel { .. }
                    | MouseInput { .. }
                    | TouchpadMagnify { .. }
                    | SmartMagnify { .. }
                    | TouchpadRotate { .. }
                    | TouchpadPressure { .. }
                    | Touch(_)
                    | ScaleFactorChanged { .. }
                    | ThemeChanged(_) => window_interaction_received = true,
                    _ => (),
                }
            }
        }

        // Process world events
        let events = self
            .world
            .get_mut::<EventQueue<WorldEvent>>()
            .receive(&self.world_event_receiver);
        for event in events {
            if let WorldEvent::EntityDestroyed(e) = event {
                self.on_entity_destroyed(e);
            }
        }

        // Process engine events
        let events = self
            .world
            .get_mut::<EventQueue<EngineEvent>>()
            .receive(&self.engine_event_receiver);
        for event in events {
            #[allow(irrefutable_let_patterns)]
            if let EngineEvent::Exit = event {
                self.on_exit();
            }
        }

        #[cfg(feature = "dbg-loop")]
        if self.timers.last_stats_display.elapsed() >= self.timers.stats_display_interval {
            self.timers.last_stats_display = Instant::now();
            info!("{}", self.world.read::<Statistics>());
        }

        #[cfg(feature = "editor")]
        if window_interaction_received {
            self.world.read::<Graphics>().request_redraw();
        }

        #[cfg(not(feature = "editor"))]
        if let Some(mld) = self.timers.min_loop_duration {
            if self.timers.last_redraw.elapsed() >= mld {
                self.world.read::<Graphics>().request_redraw();
            }
        } else {
            self.world.read::<Graphics>().request_redraw();
        }

        // Call the maintenance method of World
        if let LoopControl::Abort = self.world.maintain().await {
            event_loop_window_target.exit();
        }

        #[cfg(feature = "editor")]
        event_loop_window_target.set_control_flow(ControlFlow::wait_duration(MAX_LOOP_DURATION));
        #[cfg(not(feature = "editor"))]
        event_loop_window_target.set_control_flow(ControlFlow::Poll);
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
    /// You must supply an asynchronous Runtime so the engine can schedule tasks
    fn runtime(&self) -> Arc<Runtime>;

    /// Specifies the name of the main scene
    fn main_scene(&self) -> Option<&str> {
        None
    }

    /// Specifies the name of the asset group scenes are stored in
    fn scene_group(&self) -> &str {
        "scenes"
    }

    /// Specifies the upper bound for the duration of a loop iteration
    fn max_loop_duration(&self) -> Duration {
        MAX_LOOP_DURATION
    }

    /// Specifies the lower bound for the duration of a loop iteration
    fn min_loop_duration(&self) -> Option<Duration> {
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
    last_maintenance: Instant,
    last_redraw: Instant,
    accumulator: Duration,
    dynamic_game_time: Duration,
    fixed_game_time: Duration,
    delta_time: Duration,
    max_loop_duration: Duration,
    #[cfg(not(feature = "editor"))]
    min_loop_duration: Option<Duration>,
    #[cfg(feature = "dbg-loop")]
    last_stats_display: Instant,
    #[cfg(feature = "dbg-loop")]
    stats_display_interval: Duration,
}

impl Default for Timers {
    fn default() -> Self {
        Timers {
            last_maintenance: Instant::now(),
            last_redraw: Instant::now(),
            accumulator: Duration::default(),
            dynamic_game_time: Duration::default(),
            fixed_game_time: Duration::default(),
            delta_time: DELTA_TIME,
            max_loop_duration: MAX_LOOP_DURATION,
            #[cfg(not(feature = "editor"))]
            min_loop_duration: MIN_LOOP_DURATION,
            #[cfg(feature = "dbg-loop")]
            last_stats_display: Instant::now(),
            #[cfg(feature = "dbg-loop")]
            stats_display_interval: STATS_DISPLAY_INTERVAL,
        }
    }
}
