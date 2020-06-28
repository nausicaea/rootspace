use anyhow::{Context, Result};
use crate::{
    components::{Camera, Info, Status, Renderable, Model, UiModel},
    event::EngineEvent,
    file_manipulation::VerifyPath,
    graphics::BackendTrait,
    resources::{BackendResource, BackendSettings, SceneGraph},
    systems::{
        CameraManager, DebugConsole, DebugShell, EventCoordinator, EventInterface, EventMonitor, ForceShutdown,
        Renderer,
    },
    text_manipulation::tokenize,
};
use ecs::{
    Component, Entity, EventQueue, LoopStage, ReceiverId, RegAdd, ResourceRegistry, Resource,
    System, World, WorldEvent,
};
#[cfg(any(test, debug_assertions))]
use log::debug;
use log::trace;
use std::{
    cmp,
    marker::PhantomData,
    path::Path,
    time::{Duration, Instant},
};

pub type JoinedRegistry<RR> = RegAdd![
    <Info as Component>::Storage,
    <Status as Component>::Storage,
    <Camera as Component>::Storage,
    <Renderable as Component>::Storage,
    <UiModel as Component>::Storage,
    <Model as Component>::Storage,
    SceneGraph<UiModel>,
    SceneGraph<Model>,
    EventQueue<EngineEvent>,
    BackendSettings,
    RR
];

pub struct Orchestrator<B, RR> {
    pub world: World<JoinedRegistry<RR>>,
    delta_time: Duration,
    max_frame_time: Duration,
    world_receiver: ReceiverId<WorldEvent>,
    _b: PhantomData<B>,
}

impl<B, RR> Orchestrator<B, RR>
where
    B: BackendTrait,
    RR: ResourceRegistry,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        command: Option<&str>,
    ) -> Result<Self> {
        resource_path.ensure_extant_directory()?;

        // Create the backend
        let backend = BackendSettings::new("Title", (800, 600), true, 4, resource_path.as_ref())
            .build::<B>()
            .context("Failed to initialise the backend")?;

        // Create the world
        let mut world = World::default();

        // Insert basic resources
        world.insert(backend.settings().clone());
        world.insert(backend);

        // Insert basic systems
        world.add_system(LoopStage::Update, ForceShutdown::default());
        world.add_system(LoopStage::Update, DebugConsole::new(std::io::stdin(), Some('\\'), Some('"'), &[';']));
        world.add_system(LoopStage::Update, EventInterface::<B>::default());

        let event_monitor = EventMonitor::<WorldEvent>::new(world.get_mut::<EventQueue<WorldEvent>>());
        world.add_system(LoopStage::Update, event_monitor);

        let renderer = Renderer::<B>::new([0.69, 0.93, 0.93, 1.0], world.get_mut::<EventQueue<WorldEvent>>());
        world.add_system(LoopStage::Render, renderer);

        let event_monitor = EventMonitor::<EngineEvent>::new(world.get_mut::<EventQueue<EngineEvent>>());
        world.add_system(LoopStage::Update, event_monitor);

        let camera_manager = CameraManager::new(world.get_mut::<EventQueue<EngineEvent>>());
        world.add_system(LoopStage::Update, camera_manager);

        let debug_shell = DebugShell::new(world.get_mut::<EventQueue<EngineEvent>>(), Some(';'));
        world.add_system(LoopStage::Update, debug_shell);

        let event_coordinator = EventCoordinator::new(world.get_mut::<EventQueue<EngineEvent>>());
        world.add_system(LoopStage::Update, event_coordinator);

        trace!("Orchestrator<B, RR> subscribing to EventQueue<WorldEvent>");
        let world_receiver = world.get_mut::<EventQueue<WorldEvent>>().subscribe();

        // Send the requested debug command
        if let Some(cmd) = command {
            world.get_mut::<EventQueue<EngineEvent>>()
                .send(EngineEvent::Command(tokenize(cmd, '\\', '"', &[';'])));
        }

        Ok(Orchestrator {
            world,
            delta_time: Duration::from_millis(50),
            max_frame_time: Duration::from_millis(250),
            world_receiver,
            _b: PhantomData::default(),
        })
    }

    pub fn run(&mut self) {
        // Send the startup event
        self.world
            .get_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::Startup);

        // Initialize the timers
        let mut loop_time = Instant::now();
        let mut accumulator = Duration::default();
        let mut dynamic_game_time = Duration::default();
        let mut fixed_game_time = Duration::default();

        // Run the main game loop
        loop {
            // Assess the duration of the last frame
            let frame_time = cmp::min(loop_time.elapsed(), self.max_frame_time);
            loop_time = Instant::now();
            accumulator += frame_time;
            dynamic_game_time += frame_time;

            // Call fixed update functions until the accumulated time buffer is empty
            while accumulator >= self.delta_time {
                self.world.fixed_update(&fixed_game_time, &self.delta_time);
                accumulator -= self.delta_time;
                fixed_game_time += self.delta_time;
            }

            // Call the dynamic update and render functions
            self.world.update(&dynamic_game_time, &frame_time);
            self.world.render(&dynamic_game_time, &frame_time);

            // Perform maintenance tasks (both Orchestrator and World listen for events themselves)
            if !self.maintain() {
                break;
            }
        }
    }

    pub fn insert<R>(&mut self, res: R)
    where
        R: Resource,
    {
        self.world.insert::<R>(res)
    }

    pub fn get_mut<R: Resource>(&mut self) -> &mut R {
        self.world.get_mut::<R>()
    }

    pub fn create_entity(&mut self) -> Entity {
        self.world.create_entity()
    }

    pub fn insert_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        self.world.insert_component::<C>(entity, component)
    }

    pub fn add_system<S>(&mut self, stage: LoopStage, system: S)
    where
        S: System,
    {
        self.world.add_system::<S>(stage, system)
    }

    fn maintain(&mut self) -> bool {
        let running = self.world.maintain();

        let recv = &self.world_receiver;
        let events = self.world.get_mut::<EventQueue<WorldEvent>>().receive(recv);
        if events.into_iter().any(|e| e == WorldEvent::DeserializationComplete) {
            // Reload the backend
            if !self.world.contains::<BackendResource<B>>() {
                #[cfg(any(test, debug_assertions))]
                debug!("Reloading the backend");
                #[cfg(any(test, debug_assertions))]
                let reload_mark = Instant::now();

                let backend = self
                    .world
                    .borrow_mut::<BackendSettings>()
                    .build::<B>()
                    .expect("Unable to reload the backend");
                self.world.insert(backend);

                #[cfg(any(test, debug_assertions))]
                debug!("Completed reloading the backend after {:?}", reload_mark.elapsed());
            }
        }

        running
    }
}
