use std::time::Duration;

use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use serde::de;
use serde::export::PhantomData;

use ecs::{ReceiverId, ResourceRegistry, World, WorldEvent, short_type_name};

use crate::graphics::BackendTrait;
use crate::orchestrator::Orchestrator;
use crate::orchestrator::type_registry::TypeRegistry;

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum OrchestratorField {
    World,
    DeltaTime,
    MaxFrameTime,
    WorldReceiver,
}

pub static ORCHESTRATOR_FIELDS: &'static [&'static str] = &[
    "world",
    "delta_time",
    "max_frame_time",
    "world_receiver",
];

pub struct OrchestratorVisitor<B, RR>(PhantomData<B>, PhantomData<RR>);

impl<B, RR> Default for OrchestratorVisitor<B, RR>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
{
    fn default() -> Self {
        OrchestratorVisitor(PhantomData::default(), PhantomData::default())
    }
}

impl<'de, B, RR> Visitor<'de> for OrchestratorVisitor<B, RR>
    where
        B: BackendTrait,
        RR: ResourceRegistry,
{
    type Value = Orchestrator<B, RR>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a serialized Orchestrator<{}, {}> struct", short_type_name::<B>(), short_type_name::<RR>())
    }

    fn visit_map<A>(self, mut map_access: A) -> anyhow::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut world: Option<World<TypeRegistry<RR>>> = None;
        let mut delta_time: Option<Duration> = None;
        let mut max_frame_time: Option<Duration> = None;
        let mut world_receiver: Option<ReceiverId<WorldEvent>> = None;

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
                OrchestratorField::WorldReceiver => {
                    if world_receiver.is_some() {
                        return Err(de::Error::duplicate_field("world_receiver"));
                    }
                    world_receiver = Some(map_access.next_value()?);
                },
            }
        }

        let world = world.ok_or_else(|| de::Error::missing_field("world"))?;
        let delta_time = delta_time.ok_or_else(|| de::Error::missing_field("delta_time"))?;
        let max_frame_time = max_frame_time.ok_or_else(|| de::Error::missing_field("max_frame_time"))?;
        let world_receiver = world_receiver.ok_or_else(|| de::Error::missing_field("world_receiver"))?;

        Ok(Orchestrator {
            world,
            delta_time,
            max_frame_time,
            world_receiver,
            _b: PhantomData::default(),
        })
    }
}
