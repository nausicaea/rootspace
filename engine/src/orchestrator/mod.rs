use std::{
    cmp,
    convert::TryFrom,
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
#[cfg(any(test, debug_assertions))]
use log::debug;
use log::trace;
use serde::{Deserialize, Serialize, Serializer, Deserializer, de};

use ecs::{
    Component, Entity, EventQueue, LoopControl,
    LoopStage, ReceiverId, Resource, ResourceRegistry, System, World, WorldEvent, SystemRegistry,
};
use file_manipulation::DirPathBuf;

use crate::{
    components::{Model, UiModel},
    event::EngineEvent,
    graphics::BackendTrait,
    resources::{BackendResource, BackendSettings, SceneGraph},
    systems::{
        CameraManager, DebugConsole, DebugShell, EventCoordinator, EventInterface, EventMonitor,
        ForceShutdown, Renderer,
    },
    text_manipulation::tokenize,
};

use self::type_registry::{ResourceTypes, UpdateSystemTypes, RenderSystemTypes};
use serde::ser::SerializeStruct;
use std::marker::PhantomData;
use serde::de::{Visitor, MapAccess};

mod type_registry;

pub struct Orchestrator<B, RR, SR1, SR2, SR3> {
    pub world: World<ResourceTypes<RR>, SR1, UpdateSystemTypes<B, SR2>, RenderSystemTypes<B, SR3>>,
    delta_time: Duration,
    max_frame_time: Duration,
    receiver: ReceiverId<WorldEvent>,
}

impl<B, RR, SR1, SR2, SR3> std::fmt::Debug for Orchestrator<B, RR, SR1, SR2, SR3> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Orchestrator {{ world: {:?}, delta_time: {:?}, max_frame_time: {:?}, receiver: {:?} }}",
            self.world,
            self.delta_time,
            self.max_frame_time,
            self.receiver,
        )
    }
}

impl<B, RR, SR1, SR2, SR3> Orchestrator<B, RR, SR1, SR2, SR3>
where
    B: BackendTrait,
    RR: ResourceRegistry,
    SR1: SystemRegistry,
    SR2: SystemRegistry,
    SR3: SystemRegistry,
{
    pub fn new<P: AsRef<Path>>(resource_path: P, command: Option<&str>) -> Result<Self> {
        // Create the world
        let mut world = World::default();

        // Create the backend
        let resource_path = DirPathBuf::try_from(resource_path.as_ref())?;
        let backend = BackendSettings::new("Title", (800, 600), true, 4, resource_path)
            .build::<B>()
            .context("Failed to initialise the backend")?;

        world.insert(backend.settings().clone());
        world.insert(backend);

        // Insert basic systems
        world.add_system(LoopStage::Update, ForceShutdown::default());
        world.add_system(
            LoopStage::Update,
            DebugConsole::new(std::io::stdin(), Some('\\'), Some('"'), &[';']),
        );
        world.add_system(LoopStage::Update, EventInterface::<B>::default());

        let event_monitor =
            EventMonitor::<WorldEvent>::new(world.get_mut::<EventQueue<WorldEvent>>());
        world.add_system(LoopStage::Update, event_monitor);

        let renderer = Renderer::<B>::new(
            [0.69, 0.93, 0.93, 1.0],
            world.get_mut::<EventQueue<WorldEvent>>(),
        );
        world.add_system(LoopStage::Render, renderer);

        let event_monitor =
            EventMonitor::<EngineEvent>::new(world.get_mut::<EventQueue<EngineEvent>>());
        world.add_system(LoopStage::Update, event_monitor);

        let camera_manager = CameraManager::new(world.get_mut::<EventQueue<EngineEvent>>());
        world.add_system(LoopStage::Update, camera_manager);

        let debug_shell = DebugShell::new(world.get_mut::<EventQueue<EngineEvent>>(), Some(";"));
        world.add_system(LoopStage::Update, debug_shell);

        let event_coordinator = EventCoordinator::new(world.get_mut::<EventQueue<EngineEvent>>());
        world.add_system(LoopStage::Update, event_coordinator);

        trace!("Orchestrator<B, RR> subscribing to EventQueue<WorldEvent>");
        let receiver = world.get_mut::<EventQueue<WorldEvent>>().subscribe();

        // Send the requested debug command
        if let Some(cmd) = command {
            world
                .get_mut::<EventQueue<EngineEvent>>()
                .send(EngineEvent::Command(tokenize(cmd, '\\', '"', &[';'])));
        }

        Ok(Orchestrator {
            world,
            delta_time: Duration::from_millis(50),
            max_frame_time: Duration::from_millis(250),
            receiver,
        })
    }

    pub fn run(&mut self) {
        // Update the scene graphs for the first time
        self.world
            .borrow_mut::<SceneGraph<Model>>()
            .update(&self.world.borrow_components::<Model>());
        self.world
            .borrow_mut::<SceneGraph<UiModel>>()
            .update(&self.world.borrow_components::<UiModel>());

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
            if self.maintain() == LoopControl::Abort {
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

    fn maintain(&mut self) -> LoopControl {
        let running = self.world.maintain();

        let recv = &self.receiver;
        let events = self.world.get_mut::<EventQueue<WorldEvent>>().receive(recv);
        if events
            .into_iter()
            .any(|e| e == WorldEvent::DeserializationComplete)
        {
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
                debug!(
                    "Completed reloading the backend after {:?}",
                    reload_mark.elapsed()
                );
            }
        }

        running
    }
}

impl<B, RR, SR1, SR2, SR3> Serialize for Orchestrator<B, RR, SR1, SR2, SR3>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
        SR1: SystemRegistry,
        SR2: SystemRegistry,
        SR3: SystemRegistry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Orchestrator", 4)?;
        state.serialize_field("world", &self.world)?;
        state.serialize_field("delta_time", &self.delta_time)?;
        state.serialize_field("max_frame_time", &self.max_frame_time)?;
        state.serialize_field("receiver", &self.receiver)?;
        state.end()
    }
}

impl<'de, B, RR, SR1, SR2, SR3> Deserialize<'de> for Orchestrator<B, RR, SR1, SR2, SR3>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
        SR1: SystemRegistry,
        SR2: SystemRegistry,
        SR3: SystemRegistry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "Orchestrator",
            ORCHESTRATOR_FIELDS,
            OrchestratorVisitor::default(),
        )
    }
}

const ORCHESTRATOR_FIELDS: &'static [&'static str] = &[
    "world",
    "delta_time",
    "max_frame_time",
    "receiver",
];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum OrchestratorField {
    World,
    DeltaTime,
    MaxFrameTime,
    Receiver,
}

struct OrchestratorVisitor<B, RR, SR1, SR2, SR3>(
    PhantomData<B>,
    PhantomData<RR>,
    PhantomData<SR1>,
    PhantomData<SR2>,
    PhantomData<SR3>,
);

impl<B, RR, SR1, SR2, SR3> Default for OrchestratorVisitor<B, RR, SR1, SR2, SR3> {
    fn default() -> Self {
        OrchestratorVisitor(
            PhantomData::default(),
            PhantomData::default(),
            PhantomData::default(),
            PhantomData::default(),
            PhantomData::default(),
        )
    }
}

impl<'de, B, RR, SR1, SR2, SR3> Visitor<'de> for OrchestratorVisitor<B, RR, SR1, SR2, SR3>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
        SR1: SystemRegistry,
        SR2: SystemRegistry,
        SR3: SystemRegistry,
{
    type Value = Orchestrator<B, RR, SR1, SR2, SR3>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a serialized Orchestrator struct")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut world: Option<World<ResourceTypes<RR>, SR1, UpdateSystemTypes<B, SR2>, RenderSystemTypes<B, SR3>>> = None;
        let mut delta_time: Option<Duration> = None;
        let mut max_frame_time: Option<Duration> = None;
        let mut receiver: Option<ReceiverId<WorldEvent>> = None;

        while let Some(field_name) = map_access.next_key()? {
            match field_name {
                OrchestratorField::World => {
                    if world.is_some() {
                        return Err(de::Error::duplicate_field("world"));
                    }
                    world = Some(map_access.next_value()?);
                },
                OrchestratorField::DeltaTime => {
                    if delta_time.is_some() {
                        return Err(de::Error::duplicate_field("delta_time"));
                    }
                    delta_time = Some(map_access.next_value()?);
                },
                OrchestratorField::MaxFrameTime => {
                    if max_frame_time.is_some() {
                        return Err(de::Error::duplicate_field("max_frame_time"));
                    }
                    max_frame_time = Some(map_access.next_value()?);
                },
                OrchestratorField::Receiver => {
                    if receiver.is_some() {
                        return Err(de::Error::duplicate_field("receiver"));
                    }
                    receiver = Some(map_access.next_value()?);
                },
            }
        }

        let world = world.ok_or_else(|| de::Error::missing_field("world"))?;
        let delta_time = delta_time.ok_or_else(|| de::Error::missing_field("delta_time"))?;
        let max_frame_time = max_frame_time.ok_or_else(|| de::Error::missing_field("max_frame_time"))?;
        let receiver = receiver.ok_or_else(|| de::Error::missing_field("receiver"))?;

        Ok(Orchestrator {
            world,
            delta_time,
            max_frame_time,
            receiver,
        })
    }
}
