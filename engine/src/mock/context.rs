use components::{DepthOrderingTrait, TransformTrait};
use context::SceneGraphTrait;
use ecs::{Database, DatabaseError, DatabaseTrait, Entity, EventManagerTrait, EventTrait};
use failure::Error as FailureError;
use hierarchy::Hierarchy;
use std::{any::Any, collections::VecDeque, sync::RwLock};

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

    fn get<C: Any>(&self, entity: &Entity) -> Result<&C, DatabaseError> {
        self.database.get::<C>(entity)
    }

    fn get_mut<C: Any>(&mut self, entity: &Entity) -> Result<&mut C, DatabaseError> {
        self.database.get_mut::<C>(entity)
    }

    fn find<C: Any>(&self) -> Result<&C, DatabaseError> {
        self.database.find::<C>()
    }
}

impl<E, M> SceneGraphTrait<Entity, M> for MockCtx<E, M>
where
    E: EventTrait,
    M: Clone + Default + DepthOrderingTrait + TransformTrait + 'static,
{
    fn update_graph(&mut self) -> Result<(), FailureError> {
        self.update_graph_calls += 1;

        let db = &self.database;
        self.scene_graph.update(&|entity, _, parent_model| {
            let camera: &<M as TransformTrait>::Camera = db.find().ok()?;
            let current_model = db.get(entity).ok()?;
            parent_model.transform(camera, current_model)
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
