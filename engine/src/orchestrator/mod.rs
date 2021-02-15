use std::{
    cmp,
    time::{Duration, Instant},
};
use std::marker::PhantomData;

use anyhow::{Context, Result};
use log::debug;
use log::trace;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;

use ecs::{EventQueue, LoopControl, LoopStage, ReceiverId, ResourceRegistry, SystemRegistry, World, WorldEvent};

use crate::{
    components::{Model, UiModel},
    event::EngineEvent,
    graphics::BackendTrait,
    resources::{GraphicsBackend, SceneGraph},
    systems::DebugConsole,
};
use crate::resources::settings::Settings;

use self::type_registry::{RenderSystemTypes, ResourceTypes, UpdateSystemTypes};

mod type_registry;

pub struct Orchestrator<B, RR, FUSR, USR, RSR> {
    pub world: World<ResourceTypes<RR>, FUSR, UpdateSystemTypes<B, USR>, RenderSystemTypes<B, RSR>>,
    delta_time: Duration,
    max_frame_time: Duration,
    receiver: ReceiverId<WorldEvent>,
}

impl<B, RR, FUSR, USR, RSR> std::fmt::Debug for Orchestrator<B, RR, FUSR, USR, RSR> {
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

impl<B, RR, FUSR, USR, RSR> Orchestrator<B, RR, FUSR, USR, RSR>
where
    B: BackendTrait,
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    pub fn new(settings: Settings, command: Option<&str>) -> Result<Self> {
        // Create the graphics_backend
        // FIXME: This is the only resource that cannot be created easily. Find an alternative
        let backend = GraphicsBackend::<B>::new(&settings)
            .context("Failed to initialise the graphics_backend")?;

        // Create the world
        let mut world = World::with_settings(settings.clone());

        // Insert the backend as a resource
        world.insert(backend);

        // Subscribe to the WorldEvent queue
        let receiver = world.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>();

        // Send the requested debug command
        if let Some(cmd) = command {
            world.get_system::<DebugConsole>(LoopStage::Update)
                .send_command(cmd, world.resources());
        }

        Ok(Orchestrator {
            world,
            delta_time: settings.delta_time,
            max_frame_time: settings.max_frame_time,
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

    fn maintain(&mut self) -> LoopControl {
        let running = self.world.maintain();

        let recv = &self.receiver;
        let events = self.world.get_mut::<EventQueue<WorldEvent>>().receive(recv);
        let deser_event = events.into_iter().any(|e| e == WorldEvent::DeserializationComplete);
        if deser_event || !self.world.contains::<GraphicsBackend<B>>() {
            // Reload the graphics_backend
            debug!("Reloading the graphics_backend");
            let reload_mark = Instant::now();

            let backend = GraphicsBackend::<B>::new(&*self.world.borrow::<Settings>())
                .expect("Unable to reload the graphics_backend");
            self.world.insert(backend);

            debug!(
                "Completed reloading the graphics_backend after {:?}",
                reload_mark.elapsed()
            );
        }

        running
    }
}

impl<B, RR, FUSR, USR, RSR> Serialize for Orchestrator<B, RR, FUSR, USR, RSR>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
        FUSR: SystemRegistry,
        USR: SystemRegistry,
        RSR: SystemRegistry,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let mut state = serializer.serialize_struct("Orchestrator", 4)?;
        state.serialize_field("world", &self.world)?;
        state.serialize_field("delta_time", &self.delta_time)?;
        state.serialize_field("max_frame_time", &self.max_frame_time)?;
        state.serialize_field("receiver", &self.receiver)?;
        state.skip_field("_s")?;
        state.end()
    }
}

impl<'de, B, RR, FUSR, USR, RSR> Deserialize<'de> for Orchestrator<B, RR, FUSR, USR, RSR>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
        FUSR: SystemRegistry,
        USR: SystemRegistry,
        RSR: SystemRegistry,
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

struct OrchestratorVisitor<B, RR, FUSR, USR, RSR>(
    PhantomData<B>,
    PhantomData<RR>,
    PhantomData<FUSR>,
    PhantomData<USR>,
    PhantomData<RSR>,
);

impl<B, RR, FUSR, USR, RSR> Default for OrchestratorVisitor<B, RR, FUSR, USR, RSR> {
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

impl<'de, B, RR, FUSR, USR, RSR> Visitor<'de> for OrchestratorVisitor<B, RR, FUSR, USR, RSR>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
        FUSR: SystemRegistry,
        USR: SystemRegistry,
        RSR: SystemRegistry,
{
    type Value = Orchestrator<B, RR, FUSR, USR, RSR>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a serialized Orchestrator struct")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut world: Option<World<ResourceTypes<RR>, FUSR, UpdateSystemTypes<B, USR>, RenderSystemTypes<B, RSR>>> = None;
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
