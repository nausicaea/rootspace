use std::{
    cell::{Ref, RefMut},
    convert::TryFrom,
    fs::File,
    marker::PhantomData,
    path::Path,
    time::Duration,
};

use serde::{
    Deserialize,
    Serialize,
    de,
    ser::{self, SerializeStruct},
};
use serde_json;

use file_manipulation::{FilePathBuf, NewOrExFilePathBuf};

use crate::{
    component::Component,
    entities::Entities,
    entity::entity::Entity,
    event_queue::{EventQueue, ReceiverId},
    loop_stage::LoopStage,
    registry::ResourceRegistry,
    resource::Resource,
    resources::Resources,
    storage::Storage,
    system::System,
    systems::Systems,
    loop_control::LoopControl,
};

use crate::resources::typed_resources::TypedResources;

use self::{error::WorldError, event::WorldEvent, type_registry::ResourceTypes};
use crate::systems::typed_systems::TypedSystems;
use crate::registry::SystemRegistry;
use serde::de::{Visitor, MapAccess};

pub mod error;
pub mod event;
pub(crate) mod type_registry;

/// A World must perform actions for four types of calls that each allow a subset of the registered
/// systems to operate on the stored resources, components and entities.
pub struct World<RR, FUSR, USR, RSR> {
    resources: Resources,
    fixed_update_systems: Systems,
    update_systems: Systems,
    render_systems: Systems,
    receiver: ReceiverId<WorldEvent>,
    _rr: PhantomData<RR>,
    _sr1: PhantomData<FUSR>,
    _sr2: PhantomData<USR>,
    _sr3: PhantomData<RSR>,
}

impl<RR, FUSR, USR, RSR> World<RR, FUSR, USR, RSR>
where
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    pub fn with_settings<R: Resource>(settings: R) -> Self {
        let mut resources = Resources::with_registry::<ResourceTypes<RR>>();
        resources.insert(settings);

        let fixed_update_systems = Systems::with_registry::<FUSR>(&resources);
        let update_systems = Systems::with_registry::<USR>(&resources);
        let render_systems = Systems::with_registry::<RSR>(&resources);

        let receiver = resources.get_mut::<EventQueue<WorldEvent>>()
            .subscribe::<Self>();

        World {
            resources,
            fixed_update_systems,
            update_systems,
            render_systems,
            receiver,
            _rr: PhantomData::default(),
            _sr1: PhantomData::default(),
            _sr2: PhantomData::default(),
            _sr3: PhantomData::default(),
        }
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    /// Insert a new resource.
    pub fn insert<R>(&mut self, res: R)
    where
        R: Resource,
    {
        self.resources.insert(res)
    }

    /// Removes the resource of the specified type.
    pub fn remove<R>(&mut self)
    where
        R: Resource,
    {
        self.resources.remove::<R>()
    }

    /// Returns `true` if a resource of the specified type is present.
    pub fn contains<R>(&self) -> bool
    where
        R: Resource,
    {
        self.resources.contains::<R>()
    }

    /// Retrieves a mutable reference to a resource in the world
    pub fn get_mut<R>(&mut self) -> &mut R
    where
        R: Resource,
    {
        self.resources.get_mut::<R>()
    }

    /// Borrows the requested resource.
    pub fn borrow<R>(&self) -> Ref<R>
    where
        R: Resource,
    {
        self.resources.borrow::<R>()
    }

    /// Mutably borrows the requested resource (with a runtime borrow check).
    pub fn borrow_mut<R>(&self) -> RefMut<R>
    where
        R: Resource,
    {
        self.resources.borrow_mut::<R>()
    }

    /// Create a new `Entity`.
    pub fn create_entity(&mut self) -> Entity {
        self.resources.get_mut::<Entities>().create()
    }

    /// Add a component to the specified `Entity`.
    pub fn insert_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        self.resources
            .get_mut::<C::Storage>()
            .insert(entity, component);
    }

    pub fn borrow_components<C>(&self) -> Ref<C::Storage>
    where
        C: Component,
    {
        self.resources.borrow_components::<C>()
    }

    /// Add the specified system to the specified loop stage.
    pub fn add_system<S>(&mut self, stage: LoopStage, system: S)
    where
        S: System,
    {
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.insert(system),
            LoopStage::Update => self.update_systems.insert(system),
            LoopStage::Render => self.render_systems.insert(system),
        }
    }

    pub fn get_system<S>(&self, stage: LoopStage) -> &S
    where
        S: System,
    {
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.get::<S>(),
            LoopStage::Update => self.update_systems.get::<S>(),
            LoopStage::Render => self.render_systems.get::<S>(),
        }
    }

    pub fn get_system_mut<S>(&mut self, stage: LoopStage) -> &mut S
        where
            S: System,
    {
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.get_mut::<S>(),
            LoopStage::Update => self.update_systems.get_mut::<S>(),
            LoopStage::Render => self.render_systems.get_mut::<S>(),
        }
    }

    /// Try to retrieve the specified system type.
    pub fn find_system<S>(&self, stage: LoopStage) -> Option<&S>
    where
        S: System,
    {
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.find::<S>(),
            LoopStage::Update => self.update_systems.find::<S>(),
            LoopStage::Render => self.render_systems.find::<S>(),
        }
    }

    /// Try to retrieve the specified system type as a mutable reference.
    pub fn find_system_mut<S>(&mut self, stage: LoopStage) -> Option<&mut S>
    where
        S: System,
    {
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.find_mut::<S>(),
            LoopStage::Update => self.update_systems.find_mut::<S>(),
            LoopStage::Render => self.render_systems.find_mut::<S>(),
        }
    }

    /// The fixed update method is supposed to be called from the main loop at fixed time
    /// intervals.
    ///
    /// # Arguments
    ///
    /// * `t` - Interpreted as the current game time.
    /// * `dt` - Interpreted as the time interval between calls to `fixed_update`.
    pub fn fixed_update(&mut self, t: &Duration, dt: &Duration) {
        for system in self.fixed_update_systems.iter_mut() {
            system.run(&self.resources, t, dt);
        }
    }

    /// The dynamic update method is supposed to be called from the main loop just before the
    /// render call.
    ///
    /// # Arguments
    ///
    /// * `t` - Interpreted as the current game time.
    /// * `dt` - Interpreted as the time interval between calls to `update`.
    pub fn update(&mut self, t: &Duration, dt: &Duration) {
        for system in self.update_systems.iter_mut() {
            system.run(&self.resources, t, dt);
        }
    }

    /// The render method is supposed to be called when a re-draw of the graphical representation
    /// is desired.
    ///
    /// # Arguments
    ///
    /// * `t` - Interpreted as the current game time.
    /// * `dt` - Interpreted as the time interval between calls to `render`.
    pub fn render(&mut self, t: &Duration, dt: &Duration) {
        for system in self.render_systems.iter_mut() {
            system.run(&self.resources, t, dt);
        }
    }

    /// This method is supposed to be called when pending events or messages should be
    /// handled by the world. If this method returns
    /// [`LoopControl::Continue`](crate::loop_control::LoopControl), the execution of the
    /// main loop shall continue, otherwise it shall abort.
    pub fn maintain(&mut self) -> LoopControl {
        // Receive all pending events
        let events = self.resources.get_mut::<EventQueue<WorldEvent>>()
            .receive(&self.receiver);

        // Process all pending events
        for e in events {
            match e {
                WorldEvent::Abort => {
                    return LoopControl::Abort;
                }
                WorldEvent::Serialize(p) => self.on_serialize(&p).unwrap(),
                WorldEvent::Deserialize(p) => self.on_deserialize(&p).unwrap(),
                _ => (),
            }
        }

        LoopControl::Continue
    }

    fn on_serialize(&mut self, path: &Path) -> Result<(), WorldError> {
        // Create the serializer
        // FIXME: Find a solution not to hard-code the Serializer type
        let file_path = NewOrExFilePathBuf::try_from(path)?;
        let mut file = File::create(file_path).map_err(|e| WorldError::IoError(path.into(), e))?;
        let mut s = serde_json::Serializer::pretty(&mut file);

        // Serialize the entire World
        let _status = self.serialize(&mut s)
            .map_err(|e| WorldError::JsonError(path.into(), e))?;

        // Notify the world of the serialization event
        self.resources
            .get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::SerializationComplete);

        Ok(())
    }

    fn on_deserialize(&mut self, path: &Path) -> Result<(), WorldError> {
        // Create the deserializer
        // FIXME: Find a solution not to hard-code the Deserializer type
        let file_path = FilePathBuf::try_from(path)?;
        let mut file = File::open(&file_path).map_err(|e| WorldError::IoError(path.into(), e))?;
        let mut d = serde_json::Deserializer::from_reader(&mut file);

        // Deserialize the entire world
        let world: World<RR, FUSR, USR, RSR> = World::deserialize(&mut d)
            .map_err(|e| WorldError::JsonError(path.into(), e))?;

        // Assign its parts to the current instance
        self.resources = world.resources;
        self.fixed_update_systems = world.fixed_update_systems;
        self.update_systems = world.update_systems;
        self.render_systems = world.render_systems;
        self.receiver = world.receiver;

        // Notify the world of the deserialization event
        self.resources
            .get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::DeserializationComplete);

        Ok(())
    }
}

impl<RR, FUSR, USR, RSR> std::fmt::Debug for World<RR, FUSR, USR, RSR> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "World {{ resources: {:?}, fixed_update_systems: {:?}, update_systems: {:?}, render_systems: {:?}, receiver: {:?} }}",
            self.resources,
            self.fixed_update_systems,
            self.update_systems,
            self.render_systems,
            self.receiver,
        )
    }
}


impl<RR, FUSR, USR, RSR> Serialize for World<RR, FUSR, USR, RSR>
where
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let tr: TypedResources<'_, ResourceTypes<RR>> = (&self.resources).into();
        let ts1: TypedSystems<'_, FUSR> = (&self.fixed_update_systems).into();
        let ts2: TypedSystems<'_, USR> = (&self.update_systems).into();
        let ts3: TypedSystems<'_, RSR> = (&self.render_systems).into();

        let mut state = serializer.serialize_struct("World", WORLD_FIELDS.len())?;
        state.serialize_field("resources", &tr)?;
        state.serialize_field("fixed_update_systems", &ts1)?;
        state.serialize_field("update_systems", &ts2)?;
        state.serialize_field("render_systems", &ts3)?;
        state.serialize_field("receiver", &self.receiver)?;
        state.skip_field("_rr")?;
        state.skip_field("_sr1")?;
        state.skip_field("_sr2")?;
        state.skip_field("_sr3")?;
        state.end()
    }
}

impl<'de, RR, FUSR, USR, RSR> Deserialize<'de> for World<RR, FUSR, USR, RSR>
where
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_struct("World", WORLD_FIELDS, WorldVisitor::<RR, FUSR, USR, RSR>::default())
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum WorldField {
    Resources,
    FixedUpdateSystems,
    UpdateSystems,
    RenderSystems,
    Receiver,
}

const WORLD_FIELDS: &'static [&'static str] = &[
    "resources",
    "fixed_update_systems",
    "update_systems",
    "render_systems",
    "receiver",
];

struct WorldVisitor<RR, FUSR, USR, RSR>(PhantomData<RR>, PhantomData<FUSR>, PhantomData<USR>, PhantomData<RSR>);

impl<RR, FUSR, USR, RSR> Default for WorldVisitor<RR, FUSR, USR, RSR> {
    fn default() -> Self {
        WorldVisitor(PhantomData::default(), PhantomData::default(), PhantomData::default(), PhantomData::default())
    }
}

impl<'de, RR, FUSR, USR, RSR> Visitor<'de> for WorldVisitor<RR, FUSR, USR, RSR>
where
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    type Value = World<RR, FUSR, USR, RSR>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a serialized World struct")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
    {
        let mut resources: Option<TypedResources<ResourceTypes<RR>>> = None;
        let mut fixed_update_systems: Option<TypedSystems<FUSR>> = None;
        let mut update_systems: Option<TypedSystems<USR>> = None;
        let mut render_systems: Option<TypedSystems<RSR>> = None;
        let mut receiver: Option<ReceiverId<WorldEvent>> = None;

        while let Some(field_name) = map_access.next_key()? {
            match field_name {
                WorldField::Resources => {
                    if resources.is_some() {
                        return Err(de::Error::duplicate_field("resources"));
                    }
                    resources = Some(map_access.next_value()?);
                },
                WorldField::FixedUpdateSystems => {
                    if fixed_update_systems.is_some() {
                        return Err(de::Error::duplicate_field("fixed_update_systems"));
                    }
                    fixed_update_systems = Some(map_access.next_value()?);
                },
                WorldField::UpdateSystems => {
                    if update_systems.is_some() {
                        return Err(de::Error::duplicate_field("update_systems"));
                    }
                    update_systems = Some(map_access.next_value()?);
                },
                WorldField::RenderSystems => {
                    if render_systems.is_some() {
                        return Err(de::Error::duplicate_field("render_systems"));
                    }
                    render_systems = Some(map_access.next_value()?);
                },
                WorldField::Receiver => {
                    if receiver.is_some() {
                        return Err(de::Error::duplicate_field("receiver"));
                    }
                    receiver = Some(map_access.next_value()?);
                },
            }
        }

        let resources = resources.ok_or_else(|| de::Error::missing_field("resources"))?;
        let fixed_update_systems = fixed_update_systems.ok_or_else(|| de::Error::missing_field("fixed_update_systems"))?;
        let update_systems = update_systems.ok_or_else(|| de::Error::missing_field("update_systems"))?;
        let render_systems = render_systems.ok_or_else(|| de::Error::missing_field("render_systems"))?;
        let receiver = receiver.ok_or_else(|| de::Error::missing_field("receiver"))?;

        Ok(World {
            resources: resources.into(),
            fixed_update_systems: fixed_update_systems.into(),
            update_systems: update_systems.into(),
            render_systems: render_systems.into(),
            receiver,
            _rr: PhantomData::default(),
            _sr1: PhantomData::default(),
            _sr2: PhantomData::default(),
            _sr3: PhantomData::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Reg, VecStorage};
    use serde_test::{assert_ser_tokens, Token};

    use super::*;

    pub type Trreg = Reg![VecStorage<usize>,];

    #[test]
    fn serde() {
        let world = World::<Trreg, Reg![], Reg![], Reg![]>::with_settings(());

        assert_ser_tokens(
            &world,
            &[
                Token::Struct {
                    name: "World",
                    len: 5,
                },
                Token::Str("resources"),
                Token::Map {
                    len: Some(4),
                },
                Token::Str("Entities"),
                Token::Struct { name: "Entities", len: 3},
                Token::Str("max_idx"),
                Token::U32(0),
                Token::Str("free_idx"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("generations"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Str("EventQueue<WorldEvent>"),
                Token::Struct { name: "EventQueue", len: 4 },
                Token::Str("events"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("receivers"),
                Token::Map { len: Some(1) },
                Token::U64(0),
                Token::Struct { name: "ReceiverState", len: 3 },
                Token::Str("read"),
                Token::U64(0),
                Token::Str("received"),
                Token::U64(0),
                Token::Str("receiver_id"),
                Token::Str("World<Element<VecStorage<usize>, End>, End, End, End>"),
                Token::StructEnd,
                Token::MapEnd,
                Token::Str("max_id"),
                Token::U64(1),
                Token::Str("free_ids"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::Str("VecStorage<usize>"),
                Token::Map {
                    len: Some(0),
                },
                Token::MapEnd,
                Token::MapEnd,
                Token::Str("fixed_update_systems"),
                Token::Map {
                    len: Some(0),
                },
                Token::MapEnd,
                Token::Str("update_systems"),
                Token::Map {
                    len: Some(0),
                },
                Token::MapEnd,
                Token::Str("render_systems"),
                Token::Map {
                    len: Some(0),
                },
                Token::MapEnd,
                Token::Str("receiver"),
                Token::Struct { name: "ReceiverId", len: 1 },
                Token::Str("id"),
                Token::U64(0),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
