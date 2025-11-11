use std::{sync::Arc, time::Duration};

use futures::{StreamExt, stream::FuturesUnordered};
use parking_lot::MappedRwLockReadGuard;
use tracing::Instrument;

use self::{event::WorldEvent, type_registry::ResourceTypes};
use super::{
    component::Component,
    entities::Entities,
    entity::Entity,
    event_queue::{EventQueue, receiver_id::ReceiverId},
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
pub mod type_registry;

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
    #[tracing::instrument(skip_all)]
    pub async fn with_dependencies<RR, FUSR, USR, RS, MS, D>(deps: &D) -> Result<Self, anyhow::Error>
    where
        D: std::fmt::Debug,
        RR: ResourceRegistry + WithDependencies<D>,
        FUSR: SystemRegistry + WithResources,
        USR: SystemRegistry + WithResources,
        RS: System + WithResources,
        MS: SystemRegistry + WithResources,
    {
        let mut resources = Resources::with_dependencies::<ResourceTypes<RR>, D>(deps).await?;

        let join_result = tokio::join! {
            Systems::with_resources::<FUSR>(&resources),
            Systems::with_resources::<USR>(&resources),
            RS::with_res(&resources),
            Systems::with_resources::<MS>(&resources),
        };

        let fixed_update_systems = join_result.0?;
        let update_systems = join_result.1?;
        let render_system = join_result.2?;
        let maintenance_systems = join_result.3?;

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

    /// Retrieves a mutable reference to a resource in the world
    pub fn get_mut<R>(&mut self) -> &mut R
    where
        R: Resource,
    {
        Arc::get_mut(&mut self.resources).unwrap().get_mut::<R>()
    }

    /// Borrows the requested resource.
    pub fn read<R>(&self) -> MappedRwLockReadGuard<'_, R>
    where
        R: Resource,
    {
        self.resources.read::<R>()
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
    #[tracing::instrument(skip_all)]
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
    #[tracing::instrument(skip_all)]
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
    #[tracing::instrument(skip_all)]
    pub async fn render(&mut self, t: Duration, dt: Duration) {
        self.render_system.run(&self.resources, t, dt).await;
    }

    /// This method is supposed to be called when pending events or messages should be
    /// handled by the world. If this method returns
    /// [`LoopControl::Continue`](crate::loop_control::LoopControl), the execution of the
    /// main loop shall continue, otherwise it shall abort.
    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip_all)]
    pub fn clear(&mut self) {
        self.maintenance_systems.clear();
        self.update_systems.clear();
        self.fixed_update_systems.clear();
        Arc::get_mut(&mut self.resources).unwrap().clear();
    }

    #[tracing::instrument(skip_all)]
    async fn run_systems_parallel(systems: &Systems, resources: &Arc<Resources>, t: Duration, dt: Duration) {
        let mut fut = systems
            .into_iter()
            .map(|s| {
                let span = tracing::info_span!("system_spawn_task");
                let r = resources.clone();
                tokio::task::spawn(async move {
                    let span = tracing::info_span!("system_acquire_lock");
                    let mut sys = s.lock().instrument(span).await;
                    let span = tracing::info_span!("system_run", system = sys.name());
                    sys.run(&r, t, dt).instrument(span).await
                })
                .instrument(span)
            })
            .collect::<FuturesUnordered<_>>();

        while let Some(()) = fut.next().await.transpose().unwrap() {}
    }

    #[tracing::instrument(skip_all)]
    fn on_create_entity(&mut self) {
        let entity = self.get_mut::<Entities>().create();
        tracing::debug!("Created the entity {}", entity.idx());
        self.get_mut::<EventQueue<WorldEvent>>()
            .send(WorldEvent::EntityCreated(entity));
    }

    #[tracing::instrument(skip_all)]
    fn on_destroy_entity(&mut self, entity: Entity) {
        self.get_mut::<Entities>().destroy(entity);
        tracing::debug!("Destroyed the entity {}", entity);
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
