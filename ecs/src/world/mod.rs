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

use self::{error::WorldError, event::WorldEvent, type_registry::TypeRegistry};
use self::deserialization::{WORLD_FIELDS, WorldVisitor};
use crate::resources::serialization::SerResources;

pub mod error;
pub mod event;
mod type_registry;
mod deserialization;

/// A World must perform actions for four types of calls that each allow a subset of the registered
/// systems to operate on the stored resources, components and entities.
pub struct World<RR> {
    resources: Resources,
    fixed_update_systems: Systems,
    update_systems: Systems,
    render_systems: Systems,
    receiver: ReceiverId<WorldEvent>,
    _rr: PhantomData<RR>,
}

impl<RR> World<RR>
where
    RR: ResourceRegistry,
{
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
        let world = World::<RR>::deserialize(&mut d)
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

impl<RR> Default for World<RR>
where
    RR: ResourceRegistry,
{
    fn default() -> Self {
        let mut resources = Resources::with_registry::<TypeRegistry<RR>>();
        let receiver = resources.get_mut::<EventQueue<WorldEvent>>()
            .subscribe();

        World {
            resources,
            fixed_update_systems: Systems::default(),
            update_systems: Systems::default(),
            render_systems: Systems::default(),
            receiver,
            _rr: PhantomData::default(),
        }
    }
}

impl<RR> Serialize for World<RR>
where
    RR: ResourceRegistry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut state = serializer.serialize_struct("World", WORLD_FIELDS.len())?;
        state.serialize_field("resources", &SerResources::<TypeRegistry<RR>>::from(&self.resources))?;
        state.skip_field("fixed_update_systems")?;
        state.skip_field("update_systems")?;
        state.skip_field("render_systems")?;
        state.serialize_field("receiver", &self.receiver)?;
        state.skip_field("_rr")?;
        state.end()
    }
}

impl<'de, RR> Deserialize<'de> for World<RR>
where
    RR: ResourceRegistry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_struct("World", WORLD_FIELDS, WorldVisitor::<RR>::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Reg, VecStorage};
    use serde_test::{assert_ser_tokens, Token};

    use super::*;

    pub type TestRegistry = Reg![VecStorage<usize>,];

    #[test]
    fn default() {
        let _: World<Reg![]> = Default::default();
    }

    #[test]
    fn serde() {
        let world = World::<TestRegistry>::default();

        assert_ser_tokens(
            &world,
            &[
                Token::Struct {
                    name: "World",
                    len: 2,
                },
                Token::Str("resources"),
                Token::Map {
                    len: Some(3),
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
                Token::Struct { name: "ReceiverState", len: 2 },
                Token::Str("read"),
                Token::U64(0),
                Token::Str("received"),
                Token::U64(0),
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
