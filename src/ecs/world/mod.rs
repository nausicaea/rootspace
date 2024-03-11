use std::sync::Arc;
use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::StreamExt;

use log::debug;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};

use self::{event::WorldEvent, type_registry::ResourceTypes};
use super::{
    component::Component,
    entities::Entities,
    entity::Entity,
    event_queue::{receiver_id::ReceiverId, EventQueue},
    loop_control::LoopControl,
    registry::{ResourceRegistry, SystemRegistry},
    resource::Resource,
    resources::Resources,
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
    resources: Arc<Resources>,
    fixed_update_systems: Systems,
    update_systems: Systems,
    render_system: Box<dyn System>,
    maintenance_systems: Systems,
    receiver: ReceiverId<WorldEvent>,
}

impl World {
    pub async fn with_dependencies<RR, FUSR, USR, RS, MS, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        RR: ResourceRegistry + WithDependencies<D>,
        FUSR: SystemRegistry + WithResources,
        USR: SystemRegistry + WithResources,
        RS: System + WithResources,
        MS: SystemRegistry + WithResources,
    {
        let mut resources = Resources::with_dependencies::<ResourceTypes<RR>, D>(deps).await?;

        let fixed_update_systems = Systems::with_resources::<FUSR>(&resources).await?;
        let update_systems = Systems::with_resources::<USR>(&resources).await?;
        let render_system = RS::with_res(&resources).await?;
        let maintenance_systems = Systems::with_resources::<MS>(&resources).await?;

        let receiver = resources.get_mut::<EventQueue<WorldEvent>>().subscribe::<Self>();

        Ok(World {
            resources: Arc::new(resources),
            fixed_update_systems,
            update_systems,
            render_system: Box::new(render_system),
            maintenance_systems,
            receiver,
        })
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
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
        Arc::get_mut(&mut self.resources).unwrap().get_mut::<R>()
    }

    /// Borrows the requested resource.
    pub fn read<R>(&self) -> MappedRwLockReadGuard<R>
    where
        R: Resource,
    {
        self.resources.read::<R>()
    }

    /// Mutably borrows the requested resource (with a runtime borrow check).
    pub fn write<R>(&self) -> MappedRwLockWriteGuard<R>
    where
        R: Resource,
    {
        self.resources.write::<R>()
    }

    pub fn get_components_mut<C>(&mut self) -> &mut C::Storage
    where
        C: Component,
    {
        Arc::get_mut(&mut self.resources).unwrap().get_components_mut::<C>()
    }

    /// The fixed update method is supposed to be called from the main loop at fixed time
    /// intervals.
    ///
    /// # Arguments
    ///
    /// * `t` - Interpreted as the current game time.
    /// * `dt` - Interpreted as the time interval between calls to `fixed_update`.
    pub async fn fixed_update(&mut self, t: Duration, dt: Duration) {
        World::run_systems_parallel(&self.fixed_update_systems, &self.resources, t, dt).await
    }

    /// The dynamic update method is supposed to be called from the main loop just before the
    /// render call.
    ///
    /// # Arguments
    ///
    /// * `t` - Interpreted as the current game time.
    /// * `dt` - Interpreted as the time interval between calls to `update`.
    pub async fn update(&mut self, t: Duration, dt: Duration) {
        World::run_systems_parallel(&self.update_systems, &self.resources, t, dt).await
    }

    /// The render method is supposed to be called when a re-draw of the graphical representation
    /// is desired.
    ///
    /// # Arguments
    ///
    /// * `t` - Interpreted as the current game time.
    /// * `dt` - Interpreted as the time interval between calls to `render`.
    pub async fn render(&mut self, t: Duration, dt: Duration) {
        self.render_system.run(&self.resources, t, dt).await;
    }

    /// This method is supposed to be called when pending events or messages should be
    /// handled by the world. If this method returns
    /// [`LoopControl::Continue`](crate::loop_control::LoopControl), the execution of the
    /// main loop shall continue, otherwise it shall abort.
    pub async fn maintain(&mut self) -> LoopControl {
        // Run all custom maintenance systems
        let dummy_time = Duration::new(0, 0);
        World::run_systems_parallel(&self.maintenance_systems, &self.resources, dummy_time, dummy_time).await;

        // Receive all pending events
        let events = Arc::get_mut(&mut self.resources)
            .unwrap()
            .get_mut::<EventQueue<WorldEvent>>()
            .receive(&self.receiver);

        // Process all pending events
        for e in events {
            match e {
                WorldEvent::Exiting => {
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
        self.update_systems.clear();
        self.fixed_update_systems.clear();
        Arc::get_mut(&mut self.resources).unwrap().clear();
    }

    async fn run_systems_parallel(systems: &Systems, resources: &Arc<Resources>, t: Duration, dt: Duration) {
        let mut fut = systems
            .into_iter()
            .map(|s| {
                let r = resources.clone();
                tokio::task::spawn(async move { s.lock().await.run(&r, t, dt).await })
            })
            .collect::<FuturesUnordered<_>>();

        while let Some(()) = fut.next().await.transpose().unwrap() {}
    }

    fn on_create_entity(&mut self) {
        let entity = self.get_mut::<Entities>().create();
        debug!("Created the entity {}", entity.idx());
        self.get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::EntityCreated(entity));
    }

    fn on_destroy_entity(&mut self, entity: Entity) {
        self.get_mut::<Entities>().destroy(entity);
        debug!("Destroyed the entity {}", entity);
        self.get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::EntityDestroyed(entity));
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "World {{ resources: {:?}, fixed_update_systems: {:?}, update_systems: {:?}, receiver: {:?} }}",
            self.resources, self.fixed_update_systems, self.update_systems, self.receiver,
        )
    }
}
