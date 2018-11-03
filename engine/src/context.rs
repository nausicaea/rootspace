use components::{model::Model, DepthOrderingTrait, TransformTrait};
use ecs::{Database, DatabaseError, DatabaseTrait, Entity, EventManagerTrait};
use event::Event;
use failure::Error;
use hierarchy::Hierarchy;
use std::{any::Any, collections::VecDeque, hash::Hash};

pub struct Context {
    events: VecDeque<Event>,
    scene_graph: Hierarchy<Entity, Model>,
    database: Database,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            events: VecDeque::default(),
            scene_graph: Hierarchy::default(),
            database: Database::default(),
        }
    }
}

impl EventManagerTrait<Event> for Context {
    fn dispatch_later(&mut self, event: Event) {
        self.events.push_back(event)
    }

    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
    where
        F: FnMut(&mut Self, &Event) -> Result<bool, Error>,
    {
        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        let mut statuses: Vec<bool> = Vec::with_capacity(tmp.len());

        for event in tmp {
            statuses.push(handler(self, &event)?);
        }

        Ok(statuses.iter().all(|s| *s))
    }
}

pub trait SceneGraphTrait<K, V>
where
    K: Clone + Default + Eq + Hash,
    V: Clone + Default + TransformTrait + DepthOrderingTrait,
{
    fn update_graph(&mut self) -> Result<(), Error>;
    fn insert_node(&mut self, entity: Entity);
    fn get_nodes(&self, sort_nodes: bool) -> Vec<(&Entity, &V)>;
    fn sort_nodes(&self, nodes: &mut [(&Entity, &V)]);
}

impl SceneGraphTrait<Entity, Model> for Context {
    fn update_graph(&mut self) -> Result<(), Error> {
        let db = &self.database;
        self.scene_graph.update(&|entity, _, parent_model| {
            let camera: &<Model as TransformTrait>::Camera = db.find().ok()?;
            let current_model = db.get(entity).ok()?;
            parent_model.transform(camera, current_model)
        })?;
        Ok(())
    }

    fn insert_node(&mut self, entity: Entity) {
        self.scene_graph.insert(entity, Default::default())
    }

    fn get_nodes(&self, sort_nodes: bool) -> Vec<(&Entity, &Model)> {
        let mut nodes = self.scene_graph.iter().collect::<Vec<_>>();

        if sort_nodes {
            self.sort_nodes(&mut nodes);
        }

        nodes
    }

    fn sort_nodes(&self, nodes: &mut [(&Entity, &Model)]) {
        nodes.sort_unstable_by_key(|(_, v)| v.depth_index());
    }
}

impl DatabaseTrait for Context {
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
