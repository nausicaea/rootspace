use anyhow::Error;
use std::{
    cell::{Ref, RefMut},
    fs::File,
    marker::PhantomData,
    time::Duration,
};

use serde::{
    de,
    ser::{self, SerializeStruct},
    Deserialize, Serialize,
};

use try_default::TryDefault;

use file_manipulation::{FilePathBuf, NewOrExFilePathBuf};

use crate::{
    component::Component,
    entities::Entities,
    entity::Entity,
    event_queue::{EventQueue, ReceiverId},
    loop_control::LoopControl,
    loop_stage::LoopStage,
    registry::ResourceRegistry,
    resource::Resource,
    resources::Resources,
    storage::Storage,
    system::System,
    systems::Systems,
};

use crate::resources::typed_resources::TypedResources;

use self::{error::WorldError, event::WorldEvent, type_registry::ResourceTypes};
use crate::{registry::SystemRegistry, systems::typed_systems::TypedSystems};
use serde::de::{MapAccess, Visitor};
use log::debug;

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
    receiver: Option<ReceiverId<WorldEvent>>,
    foreign_receiver: Option<ReceiverId<WorldEvent>>,
    _rr: PhantomData<RR>,
    _sr1: PhantomData<FUSR>,
    _sr2: PhantomData<USR>,
    _sr3: PhantomData<RSR>,
}

impl<RR, FUSR, USR, RSR> TryDefault for World<RR, FUSR, USR, RSR>
where
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    fn try_default() -> Result<Self, Error> {
        let mut resources = Resources::with_registry::<ResourceTypes<RR>>()?;

        let fixed_update_systems = Systems::with_registry::<FUSR>(&resources);
        let update_systems = Systems::with_registry::<USR>(&resources);
        let render_systems = Systems::with_registry::<RSR>(&resources);

        let receiver = Some(resources.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>());
        let foreign_receiver = Some(resources.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>());

        Ok(World {
            resources,
            fixed_update_systems,
            update_systems,
            render_systems,
            receiver,
            foreign_receiver,
            _rr: PhantomData::default(),
            _sr1: PhantomData::default(),
            _sr2: PhantomData::default(),
            _sr3: PhantomData::default(),
        })
    }
}

impl<RR, FUSR, USR, RSR> World<RR, FUSR, USR, RSR>
where
    RR: ResourceRegistry,
    FUSR: SystemRegistry,
    USR: SystemRegistry,
    RSR: SystemRegistry,
{
    pub fn empty() -> Self {
        World {
            resources: Resources::default(),
            fixed_update_systems: Systems::default(),
            update_systems: Systems::default(),
            render_systems: Systems::default(),
            receiver: None,
            foreign_receiver: None,
            _rr: PhantomData::default(),
            _sr1: PhantomData::default(),
            _sr2: PhantomData::default(),
            _sr3: PhantomData::default(),
        }
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    /// Provides access to a second event queue receiver for WorldEvents that is not used by World itself
    pub fn foreign_receiver(&self) -> Option<ReceiverId<WorldEvent>> {
        self.foreign_receiver
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
        self.resources.get_mut::<C::Storage>().insert(entity, component);
    }

    pub fn borrow_components<C>(&self) -> Ref<C::Storage>
    where
        C: Component,
    {
        self.resources.borrow_components::<C>()
    }

    pub fn get_components_mut<C>(&mut self) -> &mut C::Storage
    where
        C: Component,
    {
        self.resources.get_components_mut::<C>()
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
        if let Some(ref receiver) = self.receiver {
            // Receive all pending events
            let events = self
                .resources
                .get_mut::<EventQueue<WorldEvent>>()
                .receive(receiver);

            // Process all pending events
            for e in events {
                match e {
                    WorldEvent::Abort => {
                        return LoopControl::Abort;
                    }
                    WorldEvent::Serialize(p) => self.on_serialize(&p).unwrap(),
                    WorldEvent::Deserialize(p) => self.on_deserialize(&p).unwrap(),
                    WorldEvent::CreateEntity => self.on_create_entity(),
                    WorldEvent::DestroyEntity(e) => self.on_destroy_entity(e),
                    _ => (),
                }
            }
        }

        LoopControl::Continue
    }

    fn on_serialize(&mut self, path: &NewOrExFilePathBuf) -> Result<(), WorldError> {
        // Create the serializer
        // FIXME: Find a solution not to hard-code the Serializer type
        let mut file = File::create(path).map_err(|e| WorldError::IoError(path.into(), e))?;
        let mut s = serde_json::Serializer::pretty(&mut file);

        // Serialize the entire World
        self.serialize(&mut s)
            .map_err(|e| WorldError::JsonError(path.into(), e))?;

        Ok(())
    }

    fn on_deserialize(&mut self, path: &FilePathBuf) -> Result<(), WorldError> {
        // Create the deserializer
        // FIXME: Find a solution not to hard-code the Deserializer type
        let mut file = File::open(path).map_err(|e| WorldError::IoError(path.into(), e))?;
        let mut d = serde_json::Deserializer::from_reader(&mut file);

        // Deserialize the entire world
        let world: World<RR, FUSR, USR, RSR> =
            World::deserialize(&mut d).map_err(|e| WorldError::JsonError(path.into(), e))?;

        // Assign its parts to the current instance
        self.resources = world.resources;
        self.fixed_update_systems = world.fixed_update_systems;
        self.update_systems = world.update_systems;
        self.render_systems = world.render_systems;
        self.receiver = world.receiver;

        Ok(())
    }

    fn on_create_entity(&mut self) {
       let entity = self.resources.get_mut::<Entities>().create();
        debug!("Created the entity {}", entity);
    }

    fn on_destroy_entity(&mut self, entity: Entity) {
        debug!("Destroying the entity {}", entity);
        self.resources.get_mut::<Entities>().destroy(entity);
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
        state.serialize_field("foreign_receiver", &self.foreign_receiver)?;
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
    ForeignReceiver,
}

const WORLD_FIELDS: &[&str] = &[
    "resources",
    "fixed_update_systems",
    "update_systems",
    "render_systems",
    "receiver",
    "foreign_receiver",
];

struct WorldVisitor<RR, FUSR, USR, RSR>(PhantomData<RR>, PhantomData<FUSR>, PhantomData<USR>, PhantomData<RSR>);

impl<RR, FUSR, USR, RSR> Default for WorldVisitor<RR, FUSR, USR, RSR> {
    fn default() -> Self {
        WorldVisitor(
            PhantomData::default(),
            PhantomData::default(),
            PhantomData::default(),
            PhantomData::default(),
        )
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
        let mut typed_resources: Option<TypedResources<ResourceTypes<RR>>> = None;
        let mut fixed_update_systems: Option<TypedSystems<FUSR>> = None;
        let mut update_systems: Option<TypedSystems<USR>> = None;
        let mut render_systems: Option<TypedSystems<RSR>> = None;
        let mut receiver: Option<Option<ReceiverId<WorldEvent>>> = None;
        let mut foreign_receiver: Option<Option<ReceiverId<WorldEvent>>> = None;

        while let Some(field_name) = map_access.next_key()? {
            match field_name {
                WorldField::Resources => {
                    if typed_resources.is_some() {
                        return Err(de::Error::duplicate_field("resources"));
                    }
                    typed_resources = Some(map_access.next_value::<TypedResources<ResourceTypes<RR>>>()?);
                }
                WorldField::FixedUpdateSystems => {
                    if fixed_update_systems.is_some() {
                        return Err(de::Error::duplicate_field("fixed_update_systems"));
                    }
                    fixed_update_systems = Some(map_access.next_value::<TypedSystems<FUSR>>()?);
                }
                WorldField::UpdateSystems => {
                    if update_systems.is_some() {
                        return Err(de::Error::duplicate_field("update_systems"));
                    }
                    update_systems = Some(map_access.next_value::<TypedSystems<USR>>()?);
                }
                WorldField::RenderSystems => {
                    if render_systems.is_some() {
                        return Err(de::Error::duplicate_field("render_systems"));
                    }
                    render_systems = Some(map_access.next_value::<TypedSystems<RSR>>()?);
                }
                WorldField::Receiver => {
                    if receiver.is_some() {
                        return Err(de::Error::duplicate_field("receiver"));
                    }
                    receiver = Some(map_access.next_value::<Option<ReceiverId<WorldEvent>>>()?);
                }
                WorldField::ForeignReceiver => {
                    if foreign_receiver.is_some() {
                        return Err(de::Error::duplicate_field("foreign_receiver"));
                    }
                    foreign_receiver = Some(map_access.next_value::<Option<ReceiverId<WorldEvent>>>()?);
                }
            }
        }

        let typed_resources = typed_resources.ok_or_else(|| de::Error::missing_field("resources"))?;
        let fixed_update_systems =
            fixed_update_systems.ok_or_else(|| de::Error::missing_field("fixed_update_systems"))?;
        let update_systems = update_systems.ok_or_else(|| de::Error::missing_field("update_systems"))?;
        let render_systems = render_systems.ok_or_else(|| de::Error::missing_field("render_systems"))?;
        let receiver = receiver.flatten();
        let foreign_receiver = foreign_receiver.flatten();

        // De-type the resources, and notify the world of completed deserialization
        let mut resources: Resources = typed_resources.into();
        resources.get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::DeserializationComplete);

        Ok(World {
            resources,
            fixed_update_systems: fixed_update_systems.into(),
            update_systems: update_systems.into(),
            render_systems: render_systems.into(),
            receiver,
            foreign_receiver,
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
    #[ignore = "Fails because assert_ser_tokens is not equipped to tell me the location of the error"]
    fn serde() {
        let world = World::<Trreg, Reg![], Reg![], Reg![]>::try_default().unwrap();

        assert_ser_tokens(
            &world,
            &[
                Token::Struct { name: "World", len: 6 },
                Token::Str("resources"),
                Token::Map { len: Some(3) },
                Token::Str("Entities"),
                Token::Struct {
                    name: "Entities",
                    len: 1,
                },
                Token::Str("max_idx"),
                Token::U32(0),
                Token::StructEnd,
                Token::Str("EventQueue<WorldEvent>"),
                Token::Struct {
                    name: "EventQueue",
                    len: 2,
                },
                Token::Str("receivers"),
                Token::Map { len: Some(2) },
                Token::U64(1),
                Token::Tuple {
                    len: 2,
                },
                Token::U64(0),
                Token::U64(0),
                Token::TupleEnd,
                Token::U64(0),
                Token::Tuple {
                    len: 2,
                },
                Token::U64(0),
                Token::U64(0),
                Token::TupleEnd,
                Token::MapEnd,
                Token::Str("max_id"),
                Token::U64(2),
                Token::StructEnd,
                Token::Str("VecStorage<usize>"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::MapEnd,
                Token::Str("fixed_update_systems"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::Str("update_systems"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::Str("render_systems"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::Str("receiver"),
                Token::Some,
                Token::U64(0),
                Token::Str("foreign_receiver"),
                Token::Some,
                Token::U64(1),
                Token::StructEnd,
            ],
        );
    }
}
