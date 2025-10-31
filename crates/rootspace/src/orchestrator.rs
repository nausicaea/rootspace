use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glamour::{quat::Quat, unit::Unit, vec::Vec4};
use tokio::runtime::Runtime;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
};

use super::registry::{FUSRegistry, MSRegistry};
use crate::{
    assets::scene::{LightSource, Scene},
    components::{camera::Camera, info::Info, renderable::Renderable, transform::Transform},
    events::engine_event::EngineEvent,
    registry::{RRegistry, USRegistry},
    resources::{
        asset_database::{AssetDatabase, AssetDatabaseDeps},
        graphics::{Graphics, GraphicsDeps},
        rpc_settings::RpcDeps,
        statistics::Statistics,
    },
    systems::renderer::Renderer,
    RenderableSource,
};
use ecs::{
    entity::Entity,
    event_queue::{receiver_id::ReceiverId, EventQueue},
    loop_control::LoopControl,
    registry::{ResourceRegistry, SystemRegistry},
    resources::Resources,
    storage::Storage,
    with_dependencies::WithDependencies,
    with_resources::WithResources,
    world::{event::WorldEvent, World},
};

const DELTA_TIME: Duration = Duration::from_millis(50);
#[cfg(feature = "editor")]
const MAX_LOOP_DURATION: Duration = Duration::from_secs(2);
#[cfg(not(feature = "editor"))]
const MAX_LOOP_DURATION: Duration = Duration::from_millis(250);
const MIN_LOOP_DURATION: Option<Duration> = Some(Duration::from_millis(17));

#[derive(Debug)]
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
    #[tracing::instrument(skip_all)]
    pub async fn with_dependencies<RR, FUSR, USR, MSR, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        D: std::fmt::Debug,
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

        Self::load_builtins(world.resources()).await?;

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
    /// block on [`Orchestrator::run`](Orchestrator::run), which does
    /// the actual work.
    pub fn start(mut self) -> impl 'static + FnMut(Event<()>, &EventLoopWindowTarget<()>) {
        let rt = self.runtime.clone();

        move |event, elwt| {
            rt.block_on(self.run(event, elwt));
        }
    }

    /// Handles an event from `winit` and the operating system, by:
    /// 1. Initiating window redrawing with
    ///    [`Orchestrator::redraw`](Orchestrator::redraw)
    /// 2. Running regular maintenance with
    ///    [`Orchestrator::maintain`](Orchestrator::maintain)
    /// 3. Shutting down cleanly at the end of the engine lifecycle with
    ///    [`Orchestrator::on_exiting`](Orchestrator::on_exiting)
    #[tracing::instrument(skip_all)]
    async fn run(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        #[cfg(feature = "dbg-loop")]
        let mut draw_bottom = false;
        #[cfg(feature = "dbg-loop")]
        {
            match &event {
                Event::NewEvents(_) => {
                    tracing::trace!("⬇");
                }
                Event::AboutToWait | Event::LoopExiting => {
                    draw_bottom = true;
                }
                _ => (),
            }
            tracing::trace!("Event trace: {:?}", &event);
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
            tracing::trace!("⬆\n\n");
        }
    }

    /// Update and render the engine state:
    /// 1. Call [`World::fixed_update`](World::fixed_update) with fixed
    ///    time intervals by guaranteeing that missed updates are caught up with.
    /// 2. Call [`World::update`](World::update) once per redraw event.
    /// 3. Call [`World::render`](World::render) once per redraw event.
    /// 4. Update performance statistics in [`Statistics`](Statistics)
    #[tracing::instrument(skip_all)]
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
    /// Calls [`World::maintain`](World::maintain`) after all other events
    /// have been handled.
    #[tracing::instrument(skip_all)]
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
                    | CloseRequested
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

    #[tracing::instrument(skip_all)]
    fn on_entity_destroyed(&mut self, entity: Entity) {
        tracing::trace!("Removing entity from components");
        self.world.get_components_mut::<Camera>().remove(entity);
        self.world.get_components_mut::<Info>().remove(entity);
        self.world.get_components_mut::<Transform>().remove(entity);
        self.world.get_components_mut::<Renderable>().remove(entity);
    }

    #[tracing::instrument(skip_all)]
    fn on_exit(&mut self) {
        tracing::info!("Exit requested");
        self.world.get_mut::<EventQueue<WorldEvent>>().send(WorldEvent::Exiting);
        self.world.read::<Graphics>().request_redraw();
    }

    #[tracing::instrument(skip_all)]
    fn on_exiting(&mut self) {
        tracing::info!("Exiting");
        self.world.clear();
    }

    #[tracing::instrument(skip_all)]
    async fn load_builtins(#[allow(unused_variables)] res: &Resources) -> anyhow::Result<()> {
        #[cfg(feature = "editor")]
        Self::load_editor_builtins(res).await?;

        let mut builtins_scene = Scene::default();
        builtins_scene
            .create_entity()
            .with_info(Info {
                name: "camera-1".into(),
                ..Default::default()
            })
            .with_camera(Camera::default())
            .with_transform(Transform::look_at_rh_inv(
                [0.0, 5.0, -10.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                Vec4::y(),
            ))
            .submit();
        builtins_scene
            .create_entity()
            .with_info(Info {
                name: "light-1".into(),
                ..Default::default()
            })
            .with_light(LightSource::Reference {
                group: "models".into(),
                name: "cube.ply".into(),
                position: [2.0, 2.0, 2.0, 1.0].into(),
                color: [1.0, 1.0, 1.0, 1.0].into(),
            })
            .submit();

        const SPACE_BETWEEN: f32 = 3.0;
        const NUM_INSTANCES_PER_ROW: usize = 16;
        for i in 0..NUM_INSTANCES_PER_ROW {
            for j in 0..NUM_INSTANCES_PER_ROW {
                let x = SPACE_BETWEEN * (i as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (j as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = Vec4::new(x, 0.0, z, 0.0);

                use approx::relative_eq;
                use glamour::num::Zero;

                let (axis, angle) = if relative_eq!(position, Vec4::zero()) {
                    (Vec4::z(), 0.0)
                } else {
                    (Unit::from(position), std::f32::consts::PI / 4.0)
                };

                builtins_scene
                    .create_entity()
                    .with_info(Info {
                        name: format!("cube-{i}x{j}"),
                        ..Default::default()
                    })
                    //.with_debug_animate()
                    .with_renderable(RenderableSource::Reference {
                        group: "models".into(),
                        name: "textured-cube.ply".into(),
                    })
                    .with_transform(
                        Transform::builder()
                            .with_translation(position)
                            .with_scale(0.5)
                            .with_orientation(Quat::with_axis_angle(axis, angle))
                            .build(),
                    )
                    .submit();
            }
        }

        builtins_scene.submit(res, "builtin", "main").await?;

        Ok(())
    }

    #[cfg(feature = "editor")]
    #[tracing::instrument(skip_all)]
    async fn load_editor_builtins(res: &Resources) -> anyhow::Result<()> {
        let mut editor_scene = Scene::default();
        editor_scene
            .create_entity()
            .with_info(Info {
                name: "coordinate-diag-ortho".into(),
                ..Default::default()
            })
            .with_renderable(RenderableSource::Reference {
                group: "models".into(),
                name: "coordinate-diag.ply".into(),
            })
            .with_transform(Transform::builder().with_ui(true).with_scale(0.1).build())
            .submit();
        editor_scene
            .create_entity()
            .with_info(Info {
                name: "coordinate-diag-persp".into(),
                ..Default::default()
            })
            .with_renderable(RenderableSource::Reference {
                group: "models".into(),
                name: "coordinate-diag.ply".into(),
            })
            .with_transform(Transform::default())
            .submit();
        editor_scene.submit(res, "builtin", "editor").await?;

        Ok(())
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
        }
    }
}
