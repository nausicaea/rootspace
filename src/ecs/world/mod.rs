use std::{
    cell::{Ref, RefMut},
    time::Duration,
};

use log::debug;

use self::{event::WorldEvent, type_registry::ResourceTypes};
use super::{
    component::Component,
    entities::Entities,
    entity::Entity,
    event_queue::{receiver_id::ReceiverId, EventQueue},
    loop_control::LoopControl,
    loop_stage::LoopStage,
    registry::{ResourceRegistry, SystemRegistry},
    resource::Resource,
    resources::Resources,
    storage::Storage,
    system::System,
    systems::Systems,
    with_dependencies::WithDependencies,
    with_resources::WithResources,
};

pub mod error;
pub mod event;
pub(crate) mod type_registry;

/// A World must perform actions for four types of calls that each allow a subset of the registered
/// systems to operate on the stored resources, components and entities.
pub struct World {
    resources: Resources,
    fixed_update_systems: Systems,
    update_systems: Systems,
    render_systems: Systems,
    receiver: ReceiverId<WorldEvent>,
}

impl World {
    pub async fn with_dependencies<RR, FUSR, USR, RSR, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        RR: ResourceRegistry + WithDependencies<D>,
        FUSR: SystemRegistry + WithResources,
        USR: SystemRegistry + WithResources,
        RSR: SystemRegistry + WithResources,
    {
        let mut resources = Resources::with_dependencies::<ResourceTypes<RR>, D>(deps).await?;

        let fixed_update_systems = Systems::with_resources::<FUSR>(&resources).await?;
        let update_systems = Systems::with_resources::<USR>(&resources).await?;
        let render_systems = Systems::with_resources::<RSR>(&resources).await?;

        let receiver = resources.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>();

        Ok(World {
            resources,
            fixed_update_systems,
            update_systems,
            render_systems,
            receiver,
        })
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
        let events = self
            .resources
            .get_mut::<EventQueue<WorldEvent>>()
            .receive(&self.receiver);

        // Process all pending events
        for e in events {
            match e {
                WorldEvent::Abort => {
                    return LoopControl::Abort;
                }
                WorldEvent::CreateEntity => self.on_create_entity(),
                WorldEvent::DestroyEntity(e) => self.on_destroy_entity(e),
                _ => (),
            }
        }

        LoopControl::Continue
    }

    pub fn clear(&mut self) {
        self.render_systems.clear();
        self.update_systems.clear();
        self.fixed_update_systems.clear();
        self.resources.clear();
    }

    fn on_create_entity(&mut self) {
        let entity = self.resources.get_mut::<Entities>().create();
        debug!("Created the entity {}", entity.idx());
        self.resources
            .get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::EntityCreated(entity));
    }

    fn on_destroy_entity(&mut self, entity: Entity) {
        self.resources.get_mut::<Entities>().destroy(entity);
        debug!("Destroyed the entity {}", entity);
        self.resources
            .get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::EntityDestroyed(entity));
    }
}

impl std::fmt::Debug for World {
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
