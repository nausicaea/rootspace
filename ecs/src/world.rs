//! Provides the `WorldTrait` and the `World` which manages resources and systems.

use crate::{
    components::{Component, Storage},
    entities::{Entities, Entity},
    event_queue::{EventQueue, ReceiverId},
    loop_stage::LoopStage,
    registry::Registry,
    resource::Resource,
    resources::{Persistence, Settings, Resources},
    system::System,
    RegAdd,
};
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use serde_json;
use std::{fs::File, marker::PhantomData, path::PathBuf, time::Duration};
use typename::TypeName;
use std::cell::{Ref, RefMut};
use log::trace;

/// Exposes resource management methods.
pub trait ResourcesTrait<RR>
where
    RR: Registry,
{
    type ResourceRegistry: Registry;

    /// Clears the state of the resource manager.
    fn clear(&mut self);

    /// Insert a new resource.
    fn insert<R, S>(&mut self, res: R, settings: S)
    where
        R: Resource + TypeName,
        S: Into<Option<Settings>>;

    /// Removes the resource of the specified type.
    fn remove<R>(&mut self)
    where
        R: Resource + TypeName;

    /// Returns `true` if a resource of the specified type is present.
    fn contains<R>(&self) -> bool
    where
        R: Resource;

    /// Retrieves a mutable reference to a resource in the world
    fn get_mut<R: Resource + TypeName>(&mut self) -> &mut R;

    /// Borrows the requested resource.
    fn borrow<R: Resource + TypeName>(&self) -> Ref<R>;

    /// Mutably borrows the requested resource (with a runtime borrow check).
    fn borrow_mut<R: Resource + TypeName>(&self) -> RefMut<R>;

    /// Create a new `Entity`.
    fn create_entity(&mut self) -> Entity;

    /// Add a component to the specified `Entity`.
    fn insert_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component + TypeName,
        C::Storage: TypeName;

    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer;

    fn deserialize<'de, D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>;
}

/// A World must perform actions for four types of calls that each allow a subset of the registered
/// systems to operate on the stored resources, components and entities.
pub trait WorldTrait {
    /// Add the specified system to the specified loop stage.
    fn add_system<S>(&mut self, stage: LoopStage, system: S)
    where
        S: System;

    /// Try to retrieve the specified system type.
    fn get_system<S>(&self, stage: LoopStage) -> Option<&S>
    where
        S: System;
    /// The fixed update method is supposed to be called from the main loop at fixed time
    /// intervals.
    ///
    /// # Arguments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `fixed_update`.
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration);
    /// The dynamic update method is supposed to be called from the main loop just before the
    /// render call.
    ///
    /// # Arguments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `update`.
    fn update(&mut self, time: &Duration, delta_time: &Duration);
    /// The render method is supposed to be called when a re-draw of the graphical representation
    /// is desired.
    ///
    /// # Arguments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `render`.
    fn render(&mut self, time: &Duration, delta_time: &Duration);
    /// This method is supposed to be called when pending events or messages should be
    /// handled by the world. If this method returns `true`, the execution of the
    /// main loop shall continue, otherwise it shall abort.
    fn maintain(&mut self) -> bool;
}

/// Events defined and processed by the world itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeName, Serialize, Deserialize)]
pub enum WorldEvent {
    /// Causes the WorldTrait::maintain() method to serialize the entire world state to the given
    /// file.
    Serialize(PathBuf),
    /// Signals the completion of serialization.
    SerializationComplete,
    /// Causes the WorldTrait::maintain() method to deserialize the entire world state from the
    /// given file.
    Deserialize(PathBuf),
    /// Signals the completion of deserialization.
    DeserializationComplete,
    /// Causes the WorldTrait::maintain() method to return `false`, which should result in the game
    /// engine to abort.
    Abort,
}

/// This is the default implementation of the `WorldTrait` provided by this library.
pub struct World<RR> {
    resources: Resources,
    fixed_update_systems: Vec<Box<dyn System>>,
    update_systems: Vec<Box<dyn System>>,
    render_systems: Vec<Box<dyn System>>,
    receiver: ReceiverId<WorldEvent>,
    _rr: PhantomData<RR>,
}

impl<RR> Default for World<RR> {
    fn default() -> Self {
        let mut events: EventQueue<WorldEvent> = EventQueue::default();
        trace!("World<RR> subscribing to EventQueue<WorldEvent>");
        let receiver = events.subscribe();

        let mut resources = Resources::default();
        resources.insert(Entities::default(), Persistence::Runtime);
        resources.insert(events, Persistence::Runtime);

        World {
            resources,
            fixed_update_systems: Vec::default(),
            update_systems: Vec::default(),
            render_systems: Vec::default(),
            receiver,
            _rr: PhantomData::default(),
        }
    }
}

impl<RR> ResourcesTrait<RR> for World<RR>
where
    RR: Registry,
{
    type ResourceRegistry = RegAdd![Entities, EventQueue<WorldEvent>, RR];

    fn clear(&mut self) {
        self.resources.clear();
        self.fixed_update_systems.clear();
        self.update_systems.clear();
        self.render_systems.clear();
    }

    fn insert<R, S>(&mut self, res: R, settings: S)
    where
        R: Resource + TypeName,
        S: Into<Option<Settings>>,
    {
        self.resources.insert(res, settings)
    }

    fn remove<R>(&mut self)
    where
        R: Resource + TypeName,
    {
        self.resources.remove::<R>()
    }

    fn contains<R>(&self) -> bool
    where
        R: Resource,
    {
        self.resources.contains::<R>()
    }

    fn get_mut<R>(&mut self) -> &mut R
    where
        R: Resource + TypeName,
    {
        self.resources.get_mut::<R>()
    }

    fn borrow<R>(&self) -> Ref<R>
    where
        R: Resource + TypeName,
    {
        self.resources.borrow::<R>()
    }

    fn borrow_mut<R>(&self) -> RefMut<R>
    where
        R: Resource + TypeName,
    {
        self.resources.borrow_mut::<R>()
    }

    fn create_entity(&mut self) -> Entity {
        self.resources.get_mut::<Entities>().create()
    }

    fn insert_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component + TypeName,
        C::Storage: TypeName,
    {
        if !self.resources.contains::<C::Storage>() {
            let _ = self.resources.insert(C::Storage::default(), Persistence::None);
        }

        self.resources.get_mut::<C::Storage>().insert(entity, component);
    }

    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        self.resources.serialize::<Self::ResourceRegistry, S>(serializer)
    }

    fn deserialize<'de, D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        self.resources = Resources::deserialize::<Self::ResourceRegistry, D>(deserializer)?;
        Ok(())
    }
}

impl<RR> WorldTrait for World<RR>
where
    RR: Registry,
{
    fn add_system<S>(&mut self, stage: LoopStage, system: S)
    where
        S: System,
    {
        let sys = Box::new(system);
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.push(sys),
            LoopStage::Update => self.update_systems.push(sys),
            LoopStage::Render => self.render_systems.push(sys),
        }
    }

    fn get_system<S>(&self, stage: LoopStage) -> Option<&S>
    where
        S: System,
    {
        match stage {
            LoopStage::FixedUpdate => self
                .fixed_update_systems
                .iter()
                .filter_map(|s| s.downcast_ref::<S>())
                .last(),
            LoopStage::Update => self.update_systems.iter().filter_map(|s| s.downcast_ref::<S>()).last(),
            LoopStage::Render => self.render_systems.iter().filter_map(|s| s.downcast_ref::<S>()).last(),
        }
    }

    fn fixed_update(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.fixed_update_systems {
            system.run(&self.resources, t, dt);
        }
    }

    fn update(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.update_systems {
            system.run(&self.resources, t, dt);
        }
    }

    fn render(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.render_systems {
            system.run(&self.resources, t, dt);
        }
    }

    fn maintain(&mut self) -> bool {
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
                    self.resources.get_mut::<EventQueue<WorldEvent>>().send(WorldEvent::SerializationComplete);
                },
                WorldEvent::Deserialize(p) => {
                    let mut file = File::open(&p).expect(&format!("Could not open the file {}: ", p.display()));
                    let mut d = serde_json::Deserializer::from_reader(&mut file);
                    self.deserialize(&mut d)
                        .expect(&format!("Could not deserialize from the file {}: ", p.display()));
                    self.resources.get_mut::<EventQueue<WorldEvent>>().send(WorldEvent::DeserializationComplete);
                },
                _ => (),
            }
        }

        true
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
