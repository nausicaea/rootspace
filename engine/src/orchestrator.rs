use crate::{
    components::{Model, UiModel},
    event::EngineEvent,
    file_manipulation::{FileError, VerifyPath},
    graphics::BackendTrait,
    resources::{BackendResource, BackendSettings, SceneGraph},
};
use ecs::{
    Component, Entity, EventQueue, LoopStage, Persistence, ReceiverId, RegAdd, Registry, Resource, ResourcesTrait,
    System, WorldTrait, World, Settings, WorldEvent
};
use serde::{de::Deserializer, ser::Serializer};
use std::{
    cmp,
    marker::PhantomData,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use typename::TypeName;
#[cfg(any(test, debug_assertions))]
use log::debug;
use log::trace;

pub type JoinedRegistry<RR> = RegAdd![
    SceneGraph<UiModel>,
    SceneGraph<Model>,
    EventQueue<EngineEvent>,
    BackendSettings,
    RR
];

pub struct Orchestrator<B, RR> {
    pub world: World<JoinedRegistry<RR>>,
    resource_path: PathBuf,
    delta_time: Duration,
    max_frame_time: Duration,
    world_receiver: ReceiverId<WorldEvent>,
    _b: PhantomData<B>,
    _rr: PhantomData<RR>,
}

impl<B, RR> Orchestrator<B, RR>
where
    B: BackendTrait,
    RR: Registry,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, FileError> {
        resource_path.ensure_extant_directory()?;

        let backend = BackendSettings::new("Title", (800, 600), true, 4)
            .build::<B>()
            .expect("Failed to initialise the backend");

        let mut world = World::default();
        world.insert(EventQueue::<EngineEvent>::default(), Persistence::Runtime);
        world.insert(backend.settings().clone(), Persistence::Runtime);
        world.insert(backend, Persistence::Runtime);
        world.insert(SceneGraph::<Model>::default(), Persistence::Runtime);
        world.insert(SceneGraph::<UiModel>::default(), Persistence::Runtime);

        trace!("Orchestrator<B, RR> subscribing to EventQueue<WorldEvent>");
        let world_receiver = world.get_mut::<EventQueue<WorldEvent>>()
            .subscribe();

        Ok(Orchestrator {
            world,
            resource_path: resource_path.as_ref().into(),
            delta_time,
            max_frame_time,
            world_receiver,
            _b: PhantomData::default(),
            _rr: PhantomData::default(),
        })
    }

    pub fn run(&mut self, iterations: Option<usize>) {
        let mut loop_time = Instant::now();
        let mut accumulator = Duration::default();
        let mut dynamic_game_time = Duration::default();
        let mut fixed_game_time = Duration::default();

        let mut i = 0;
        let mut running = true;
        while running && iterations.map(|max_iter| i < max_iter).unwrap_or(true) {
            let frame_time = cmp::min(loop_time.elapsed(), self.max_frame_time);
            loop_time = Instant::now();
            accumulator += frame_time;
            dynamic_game_time += frame_time;

            while accumulator >= self.delta_time {
                self.world.fixed_update(&fixed_game_time, &self.delta_time);
                accumulator -= self.delta_time;
                fixed_game_time += self.delta_time;
            }

            self.world.update(&dynamic_game_time, &frame_time);
            self.world.render(&dynamic_game_time, &frame_time);
            running = self.maintain();
            i += 1;
        }
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
                let backend = self.world.borrow_mut::<BackendSettings>()
                    .build::<B>()
                    .expect("Unable to reload the backend");
                self.world.insert(backend, Persistence::Runtime);
                #[cfg(any(test, debug_assertions))]
                debug!("Completed reloading the backend after {:?}", reload_mark.elapsed());
            }
        }

        running
    }

    pub fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        self.world.serialize::<S>(serializer)
    }

    pub fn deserialize<'de, D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        self.world.deserialize::<D>(deserializer)
    }

    pub fn insert<R, S>(&mut self, res: R, settings: S)
    where
        R: Resource + TypeName,
        S: Into<Option<Settings>>,
    {
        self.world.insert::<R, S>(res, settings)
    }

    pub fn get_mut<R: Resource + TypeName>(&mut self) -> &mut R {
        self.world.get_mut::<R>()
    }

    pub fn create_entity(&mut self) -> Entity {
        self.world.create_entity()
    }

    pub fn insert_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component + TypeName,
        C::Storage: TypeName,
    {
        self.world.insert_component::<C>(entity, component)
    }

    pub fn add_system<S>(&mut self, stage: LoopStage, system: S)
    where
        S: System,
    {
        self.world.add_system::<S>(stage, system)
    }

    pub fn file(&self, folder: &str, file: &str) -> Result<PathBuf, FileError> {
        let path = self.resource_path.join(folder).join(file);
        path.ensure_extant_file()?;
        Ok(path)
    }
}
