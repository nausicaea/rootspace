//! Provides the `WorldTrait` and the `World` which manages resources and systems.

use crate::{
    components::{Component, Storage},
    entities::{Entities, Entity},
    event_queue::{EventQueue, ReceiverId},
    loop_stage::LoopStage,
    persistence::Persistence,
    resource::Resource,
    resources::Resources,
    system::System,
};
use std::time::Duration;
use typename::TypeName;
use hlist::{HList, Element};
use failure::Error;
use std::marker::PhantomData;
use std::path::Path;
use std::fs::File;
use serde::{Deserialize, Serialize};
use serde_json;

/// Exposes resource management methods.
pub trait ResourcesTrait {
    /// Loads all registered resources from the specified path.
    fn load_from<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error>;
    /// Saves all registered resources to the specified path.
    fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<(), Error>;
    /// Clears the state of the world. This removes all resources whose persistence is less or
    /// equal to the specified persistence value.
    fn clear(&mut self, persistence: Persistence);
    /// Adds a resource to the world
    ///
    /// # Arguments
    ///
    /// * `res` - The resource to be added.
    /// * `persistence` - How persistence the resource should be.
    fn add_resource<R: Resource + TypeName>(&mut self, res: R, persistence: Persistence);
    /// Retrieves a mutable reference to a resource in the world
    fn get_resource_mut<R: Resource + TypeName>(&mut self) -> &mut R;
    /// Create a new `Entity`.
    fn create_entity(&mut self) -> Entity;
    /// Add a component to the specified `Entity`.
    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component + TypeName,
        C::Storage: TypeName;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, TypeName, Serialize, Deserialize)]
pub enum WorldEvent {
    /// Causes the WorldTrait::maintain() method to return `false`, which should result in the game
    /// engine to abort.
    Abort,
}

/// This is the default implementation of the `WorldTrait` provided by this library.
pub struct World<H> {
    resources: Resources,
    fixed_update_systems: Vec<Box<dyn System>>,
    update_systems: Vec<Box<dyn System>>,
    render_systems: Vec<Box<dyn System>>,
    receiver: ReceiverId<WorldEvent>,
    _h: PhantomData<H>,
}

impl<H> Default for World<H> {
    fn default() -> Self {
        let mut events: EventQueue<WorldEvent> = EventQueue::default();
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
            _h: PhantomData::default(),
        }
    }
}

impl<H> ResourcesTrait for World<H>
where
    H: HList + Serialize + for<'de> Deserialize<'de>,
{
    fn load_from<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let file = File::open(path)?;
        let mut de = serde_json::Deserializer::from_reader(file);
        self.resources = Resources::deserialize::<Element<Entities, Element<EventQueue<WorldEvent>, H>>, _>(&mut de)?;
        Ok(())
    }

    fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let file = File::create(path)?;
        let mut ser = serde_json::Serializer::new(file);
        self.resources.serialize::<Element<Entities, Element<EventQueue<WorldEvent>, H>>, _>(&mut ser)?;
        Ok(())
    }

    fn clear(&mut self, persistence: Persistence) {
        self.resources.clear(persistence);
        self.fixed_update_systems.clear();
        self.update_systems.clear();
        self.render_systems.clear();
    }

    fn add_resource<R>(&mut self, res: R, persistence: Persistence)
    where
        R: Resource + TypeName,
    {
        self.resources.insert(res, persistence)
    }

    fn get_resource_mut<R>(&mut self) -> &mut R
    where
        R: Resource + TypeName,
    {
        self.resources.get_mut::<R>()
    }

    fn create_entity(&mut self) -> Entity {
        self.resources.get_mut::<Entities>().create()
    }

    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component + TypeName,
        C::Storage: TypeName,
    {
        if !self.resources.has::<C::Storage>() {
            let _ = self.resources.insert(C::Storage::default(), Persistence::None);
        }

        self.resources.get_mut::<C::Storage>().insert(entity, component);
    }
}

impl<H> WorldTrait for World<H> {
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

        !events.into_iter().any(|e| e == WorldEvent::Abort)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let _: World<()> = Default::default();
    }
}
