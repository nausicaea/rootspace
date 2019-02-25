use std::{marker::PhantomData, time::Duration};
use crate::system::{System, EventHandlerSystem};
use crate::loop_stage::LoopStage;
use crate::resources::{Resources, Resource};
use crate::entities::{Entities, Entity};
use crate::events::{EventTrait, EventManager};
use crate::components::{Storage, Component};

/// A World must perform actions for four types of calls.
pub trait WorldTrait {
    /// Clears the state of the world. This removes all resources and systems added by the user.
    fn clear(&mut self);
    /// Adds a resource to the world
    ///
    /// # Arguments
    ///
    /// * `res` - The resource to be added.
    fn add_resource<R>(&mut self, res: R) -> Option<R> where R: Resource;
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
    /// The handle events method is supposed to be called when pending events or messages should be
    /// handled by the connected systems. If this method returns `Ok(true)`, the execution of the
    /// main loop shall continue, otherwise it shall abort.
    fn handle_events(&mut self) -> bool;
}

/// This is the default implementation of the `WorldTrait` provided by this library.
pub struct World<E> {
    resources: Resources,
    fixed_update_systems: Vec<Box<System>>,
    update_systems: Vec<Box<System>>,
    render_systems: Vec<Box<System>>,
    event_handler_systems: Vec<Box<EventHandlerSystem<E>>>,
    _e: PhantomData<E>,
}

impl<E> World<E>
where
    E: EventTrait,
{
    pub fn get_resource_mut<R>(&mut self) -> Option<&mut R> where R: Resource {
        self.resources.get_mut::<R>()
    }

    pub fn add_system<S>(&mut self, stage: LoopStage, system: S) where S: System + 'static {
        let sys = Box::new(system);
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.push(sys),
            LoopStage::Update => self.update_systems.push(sys),
            LoopStage::Render => self.render_systems.push(sys),
        }
    }

    pub fn add_event_handler_system<H>(&mut self, system: H) where H: EventHandlerSystem<E> + 'static {
        self.event_handler_systems.push(Box::new(system))
    }

    pub fn create_entity(&mut self) -> Entity {
        self.resources.get_mut::<Entities>()
            .expect("Could not find the Entities resource")
            .create()
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C) -> Option<C> where C: Component {
        if !self.resources.has::<C::Storage>() {
            let _ = self.resources.insert(C::Storage::default());
        }

        self.resources.get_mut::<C::Storage>()
            .expect("Could not find the requested component storage")
            .insert(entity, component)
    }
}

impl<E> Default for World<E>
where
    E: 'static,
{
    fn default() -> Self {
        let mut resources = Resources::default();
        resources.insert(Entities::default());
        resources.insert(EventManager::<E>::default());

        World {
            resources,
            fixed_update_systems: Vec::default(),
            update_systems: Vec::default(),
            render_systems: Vec::default(),
            event_handler_systems: Vec::default(),
            _e: PhantomData::default(),
        }
    }
}

impl<E> WorldTrait for World<E>
where
    E: EventTrait,
{
    fn clear(&mut self) {
        self.resources.clear();
        self.fixed_update_systems.clear();
        self.update_systems.clear();
        self.render_systems.clear();
        self.event_handler_systems.clear();

        self.resources.insert(Entities::default());
        self.resources.insert(EventManager::<E>::default());
    }

    fn add_resource<R>(&mut self, res: R) -> Option<R> where R: Resource {
        self.resources.insert(res)
    }

    fn fixed_update(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.fixed_update_systems {
            system.run(&mut self.resources, t, dt);
        }
    }

    fn update(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.update_systems {
            system.run(&mut self.resources, t, dt);
        }
    }

    fn render(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.render_systems {
            system.run(&mut self.resources, t, dt);
        }
    }

    fn handle_events(&mut self) -> bool {
        let events = self.resources.get_mut::<EventManager<E>>()
            .filter(|mgr| mgr.len() > 0)
            .map(|mgr| mgr.flush());

        if let Some(events) = events {
            let mut statuses: Vec<bool> = Vec::with_capacity(events.len() * self.event_handler_systems.len());
            for event in &events {
                for system in self.event_handler_systems.iter_mut() {
                    if event.matches_filter(system.get_event_filter()) {
                        statuses.push(system.run(&mut self.resources, event));
                    }
                }
            }
            statuses.into_iter().all(|s| s)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEvt;

    #[test]
    fn default() {
        let _: World<MockEvt> = Default::default();
    }
}
