use components::DepthOrderingTrait;
use context::SceneGraphTrait;
use ecs::database::{Database, DatabaseTrait, Error as DatabaseError};
use ecs::entity::Entity;
use ecs::event::{EventManagerTrait, EventTrait};
use failure::Error as FailureError;
use hierarchy::Hierarchy;
use std::any::Any;
use std::collections::VecDeque;
use std::ops::Mul;
use std::sync::RwLock;

#[derive(Debug)]
pub struct MockCtx<E, M> {
    pub events: VecDeque<E>,
    pub dispatch_later_calls: usize,
    pub handle_events_calls: usize,
    pub database: Database,
    pub scene_graph: Hierarchy<Entity, M>,
    pub update_graph_calls: usize,
    pub gnc: RwLock<usize>,
}

impl<E, M> MockCtx<E, M> {
    pub fn get_nodes_calls(&self) -> usize {
        *self.gnc.read().unwrap()
    }
}

impl<E, M> Default for MockCtx<E, M>
where
    M: Clone + Default,
{
    fn default() -> Self {
        MockCtx {
            events: VecDeque::default(),
            dispatch_later_calls: 0,
            handle_events_calls: 0,
            database: Database::default(),
            scene_graph: Hierarchy::default(),
            update_graph_calls: 0,
            gnc: RwLock::default(),
        }
    }
}

impl<E, M> EventManagerTrait<E> for MockCtx<E, M>
where
    E: EventTrait,
{
    fn dispatch_later(&mut self, event: E) {
        self.dispatch_later_calls += 1;
        self.events.push_back(event)
    }
    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, FailureError>
    where
        F: FnMut(&mut Self, &E) -> Result<bool, FailureError>,
    {
        self.handle_events_calls += 1;

        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        for event in tmp {
            handler(self, &event)?;
        }

        Ok(true)
    }
}

impl<E, M> DatabaseTrait for MockCtx<E, M>
where
    M: Clone + Default,
{
    fn create_entity(&mut self) -> Entity {
        self.database.create_entity()
    }

    fn destroy_entity(&mut self, entity: &Entity) -> Result<(), DatabaseError> {
        self.database.destroy_entity(entity)
    }

    fn has_entity(&self, entity: &Entity) -> bool {
        self.database.has_entity(entity)
    }

    fn entities(&self) -> usize {
        self.database.entities()
    }

    fn add<C: Any>(&mut self, entity: Entity, component: C) -> Result<(), DatabaseError> {
        self.database.add::<C>(entity, component)
    }

    fn remove<C: Any>(&mut self, entity: &Entity) -> Result<C, DatabaseError> {
        self.database.remove(entity)
    }

    fn has<C: Any>(&self, entity: &Entity) -> bool {
        self.database.has::<C>(entity)
    }

    fn components(&self, entity: &Entity) -> usize {
        self.database.components(entity)
    }

    fn borrow<C: Any>(&self, entity: &Entity) -> Result<&C, DatabaseError> {
        self.database.borrow::<C>(entity)
    }

    fn borrow_mut<C: Any>(&mut self, entity: &Entity) -> Result<&mut C, DatabaseError> {
        self.database.borrow_mut::<C>(entity)
    }
}

impl<E, M> SceneGraphTrait<Entity, M> for MockCtx<E, M>
where
    E: EventTrait,
    M: Clone + Default + DepthOrderingTrait + 'static,
    for<'r> &'r M: Mul<Output = M>,
{
    fn update_graph(&mut self) -> Result<(), FailureError> {
        self.update_graph_calls += 1;

        let db = &self.database;
        self.scene_graph.update(&|entity, _, parent_model| {
            let current_model = db.borrow(entity).ok()?;
            Some(parent_model * current_model)
        })?;
        Ok(())
    }

    fn insert_node(&mut self, entity: Entity) {
        self.scene_graph.insert(entity, Default::default())
    }

    fn get_nodes(&self, sort_nodes: bool) -> Vec<(&Entity, &M)> {
        let mut calls = self.gnc.write().unwrap();
        *calls += 1;

        let mut nodes = self.scene_graph.iter().collect::<Vec<_>>();

        if sort_nodes {
            self.sort_nodes(&mut nodes);
        }

        nodes
    }

    fn sort_nodes(&self, nodes: &mut [(&Entity, &M)]) {
        nodes.sort_unstable_by_key(|(_, v)| v.depth_index());
    }
}
