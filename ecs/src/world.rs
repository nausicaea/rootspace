//! Provides the `WorldTrait` and the `World` which manages resources and systems.

use thiserror::Error;
use crate::{component::Component, storage::Storage, entity::entity::Entity, entities::Entities, event_queue::{EventQueue, ReceiverId}, loop_stage::LoopStage, registry::ResourceRegistry, resource::Resource, resources::{Resources, ConflictResolution}, system::System, systems::Systems, RegAdd, LoopControl};
use serde::{Deserialize, Serialize};
use serde_json;
use log::trace;
use std::{
    cell::{Ref, RefMut},
    fs::File,
    marker::PhantomData,
    path::{Path, PathBuf},
    time::Duration,
};
use file_manipulation::{NewOrExFilePathBuf, FilePathBuf, FileError};
use std::convert::TryFrom;

/// Events defined and processed by the world itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldEvent {
    /// Causes the WorldTrait::maintain() method to serialize the entire world state to the given
    /// file.
    Serialize(PathBuf),
    /// Signals the completion of serialization.
    SerializationComplete,
    /// Causes the WorldTrait::maintain() method to deserialize the entire world state from the
    /// given file.
    Deserialize(PathBuf),
    /// Causes the WorldTrait::maintain() method to deserialize a world state additively from a
    /// file into the currently loaded state.
    DeserializeAdditive(PathBuf, ConflictResolution),
    /// Signals the completion of deserialization.
    DeserializationComplete,
    /// Causes the WorldTrait::maintain() method to return `false`, which should result in the game
    /// engine to abort.
    Abort,
}

#[derive(Debug, Error)]
pub enum WorldError {
    #[error(transparent)]
    FileError(#[from] FileError),
    #[error("{}: {}", .1, .0.display())]
    IoError(PathBuf, #[source] std::io::Error),
    #[error("{}: {}", .1, .0.display())]
    JsonError(PathBuf, #[source] serde_json::Error),
}

type JoinedRegistry<RR> = RegAdd![
    Entities,
    EventQueue<WorldEvent>,
    RR
];

/// A World must perform actions for four types of calls that each allow a subset of the registered
/// systems to operate on the stored resources, components and entities.
pub struct World<RR> {
    resources: Resources,
    fixed_update_systems: Systems,
    update_systems: Systems,
    render_systems: Systems,
    receiver: ReceiverId<WorldEvent>,
    loaded_states: Vec<PathBuf>,
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
        self.resources.get_mut::<C::Storage>().insert(entity, component);
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
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `fixed_update`.
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
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `update`.
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
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `render`.
    pub fn render(&mut self, t: &Duration, dt: &Duration) {
        for system in self.render_systems.iter_mut() {
            system.run(&self.resources, t, dt);
        }
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WorldError> {
        self.on_serialize(path.as_ref())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WorldError> {
        self.on_deserialize(path.as_ref())?;
        self.maintain();
        Ok(())
    }

    pub fn load_additive<P: AsRef<Path>>(&mut self, path: P, strategy: ConflictResolution) -> Result<(), WorldError> {
        self.on_deserialize_additive(path.as_ref(), strategy)?;
        self.maintain();
        Ok(())
    }

    /// This method is supposed to be called when pending events or messages should be
    /// handled by the world. If this method returns `LoopControl::Continue`, the execution of the
    /// main loop shall continue, otherwise it shall abort.
    pub fn maintain(&mut self) -> LoopControl {
        let events = self
            .resources
            .get_mut::<EventQueue<WorldEvent>>()
            .receive(&self.receiver);

        for e in events {
            match e {
                WorldEvent::Abort => {
                    return LoopControl::Abort;
                },
                WorldEvent::Serialize(p) => self.on_serialize(&p).unwrap(),
                WorldEvent::Deserialize(p) => self.on_deserialize(&p).unwrap(),
                WorldEvent::DeserializeAdditive(p, m) => self.on_deserialize_additive(&p, m).unwrap(),
                _ => (),
            }
        }

        LoopControl::Continue
    }

    fn on_serialize(&mut self, path: &Path) -> Result<(), WorldError> {
        let file_path = NewOrExFilePathBuf::try_from(path)?;
        let mut file = File::create(file_path).map_err(|e| WorldError::IoError(path.into(), e))?;
        let mut s = serde_json::Serializer::pretty(&mut file);

        self.resources.serialize::<JoinedRegistry<RR>, _>(&mut s)
            .map_err(|e| WorldError::JsonError(path.into(), e))?;

        self.resources
            .get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::SerializationComplete);

        Ok(())
    }

    fn on_deserialize(&mut self, path: &Path) -> Result<(), WorldError> {
        let file_path = FilePathBuf::try_from(path)?;
        let mut file = File::open(file_path).map_err(|e| WorldError::IoError(path.into(), e))?;
        let mut d = serde_json::Deserializer::from_reader(&mut file);

        self.resources = Resources::deserialize::<JoinedRegistry<RR>, _>(&mut d)
            .map_err(|e| WorldError::JsonError(path.into(), e))?;

        self.loaded_states.clear();
        self.loaded_states.push(path.into());

        self.resources
            .get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::DeserializationComplete);

        Ok(())
    }

    fn on_deserialize_additive(&mut self, path: &Path, strategy: ConflictResolution) -> Result<(), WorldError> {
        let file_path = FilePathBuf::try_from(path)?;
        let mut file = File::open(file_path).map_err(|e| WorldError::IoError(path.into(), e))?;
        let mut d = serde_json::Deserializer::from_reader(&mut file);

        self.resources
            .deserialize_additive::<JoinedRegistry<RR>, _>(&mut d, strategy)
            .map_err(|e| WorldError::JsonError(path.into(), e))?;

        self.loaded_states.push(path.into());

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
        let mut resources = Resources::with_capacity(JoinedRegistry::<RR>::LEN);
        resources.initialize::<JoinedRegistry<RR>>();

        trace!("World<RR> subscribing to EventQueue<WorldEvent>");
        let receiver = resources.get_mut::<EventQueue<WorldEvent>>().subscribe();

        World {
            resources,
            fixed_update_systems: Systems::default(),
            update_systems: Systems::default(),
            render_systems: Systems::default(),
            receiver,
            loaded_states: Vec::default(),
            _rr: PhantomData::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Reg;

    #[test]
    fn default() {
        let _: World<Reg![]> = Default::default();
    }
}
