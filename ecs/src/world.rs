//! Provides the `WorldTrait` and the `World` which manages resources and systems.

use crate::{
    component::Component,
    storage::Storage,
    entity::entity::Entity,
    entities::Entities,
    event_queue::{EventQueue, ReceiverId},
    loop_stage::LoopStage,
    registry::ResourceRegistry,
    resource::Resource,
    resources::{Resources, ConflictResolution},
    system::System,
    systems::Systems,
    RegAdd,
};
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use serde_json;
use log::trace;
use std::{
    cell::{Ref, RefMut},
    fs::File,
    marker::PhantomData,
    path::PathBuf,
    time::Duration,
};

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
    _rr: PhantomData<RR>,
}

impl<RR> World<RR>
where
    RR: ResourceRegistry,
{
    /// Clears the state of the resource manager.
    pub fn clear(&mut self) {
        self.resources.clear();
        self.fixed_update_systems.clear();
        self.update_systems.clear();
        self.render_systems.clear();
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

    pub fn serialize<S>(&self, ser: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        self.resources.serialize::<JoinedRegistry<RR>, S>(ser)
    }

    pub fn deserialize<'de, D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        self.resources = Resources::deserialize::<JoinedRegistry<RR>, D>(deserializer)?;
        Ok(())
    }

    pub fn deserialize_additive<'de, D>(&mut self, deserializer: D, method: ConflictResolution) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        self.resources.deserialize_additive::<JoinedRegistry<RR>, D>(deserializer, method)?;
        Ok(())
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

    /// This method is supposed to be called when pending events or messages should be
    /// handled by the world. If this method returns `true`, the execution of the
    /// main loop shall continue, otherwise it shall abort.
    pub fn maintain(&mut self) -> bool {
        let events = self
            .resources
            .get_mut::<EventQueue<WorldEvent>>()
            .receive(&self.receiver);

        for e in events {
            match e {
                WorldEvent::Abort => {
                    return false;
                },
                WorldEvent::Serialize(p) => {
                    let mut file = File::create(&p).expect(&format!("Could not create the file {}: ", p.display()));
                    let mut s = serde_json::Serializer::pretty(&mut file);
                    self.serialize(&mut s)
                        .expect(&format!("Could not serialize to the file {}: ", p.display()));
                    self.resources
                        .get_mut::<EventQueue<WorldEvent>>()
                        .send(WorldEvent::SerializationComplete);
                },
                WorldEvent::Deserialize(p) => {
                    let mut file = File::open(&p).expect(&format!("Could not open the file {}: ", p.display()));
                    let mut d = serde_json::Deserializer::from_reader(&mut file);
                    self.deserialize(&mut d)
                        .expect(&format!("Could not deserialize from the file {}: ", p.display()));
                    self.resources
                        .get_mut::<EventQueue<WorldEvent>>()
                        .send(WorldEvent::DeserializationComplete);
                },
                WorldEvent::DeserializeAdditive(p, m) => {
                    let mut file = File::open(&p).expect(&format!("Could not open the file {}: ", p.display()));
                    let mut d = serde_json::Deserializer::from_reader(&mut file);
                    self.deserialize_additive(&mut d, m)
                        .expect(&format!("Could not deserialize additively from the file {}: ", p.display()));
                    self.resources
                        .get_mut::<EventQueue<WorldEvent>>()
                        .send(WorldEvent::DeserializationComplete);
                },
                _ => (),
            }
        }

        true
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
        let receiver = resources.borrow_mut::<EventQueue<WorldEvent>>().subscribe();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Reg;

    #[test]
    fn default() {
        let _: World<Reg![]> = Default::default();
    }
}
