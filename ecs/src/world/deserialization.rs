use std::marker::PhantomData;
use crate::registry::ResourceRegistry;
use serde::de;
use super::World;
use crate::resources::deserialization::DeResources;
use crate::event_queue::ReceiverId;
use super::event::WorldEvent;
use crate::systems::Systems;
use serde::Deserialize;
use crate::world::type_registry::TypeRegistry;
use crate::short_type_name::short_type_name;

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum WorldField {
    Resources,
    FixedUpdateSystems,
    UpdateSystems,
    RenderSystems,
    Receiver,
}

pub const WORLD_FIELDS: &'static [&'static str] = &[
    "resources",
    "receiver",
];

pub struct WorldVisitor<RR>(PhantomData<RR>);

impl<RR> Default for WorldVisitor<RR>
    where
        RR: ResourceRegistry
{
    fn default() -> Self {
        WorldVisitor(PhantomData::default())
    }
}

impl<'de, RR> de::Visitor<'de> for WorldVisitor<RR>
    where
        RR: ResourceRegistry,
{
    type Value = World<RR>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a serialized World<{}> struct", short_type_name::<RR>())
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
    {
        let mut resources: Option<DeResources<TypeRegistry<RR>>> = None;
        let mut receiver: Option<ReceiverId<WorldEvent>> = None;

        while let Some(field_name) = map_access.next_key()? {
            match field_name {
                WorldField::Resources => {
                    if resources.is_some() {
                        return Err(de::Error::duplicate_field("resources"));
                    }
                    resources = Some(map_access.next_value()?);
                },
                WorldField::Receiver => {
                    if receiver.is_some() {
                        return Err(de::Error::duplicate_field("receiver"));
                    }
                    receiver = Some(map_access.next_value()?);
                },
                _ => (),
            }
        }

        let resources = resources.ok_or_else(|| de::Error::missing_field("resources"))?;
        let receiver = receiver.ok_or_else(|| de::Error::missing_field("receiver"))?;

        Ok(World {
            resources: resources.into(),
            fixed_update_systems: Systems::default(),
            update_systems: Systems::default(),
            render_systems: Systems::default(),
            receiver,
            _rr: PhantomData::default(),
        })
    }
}

