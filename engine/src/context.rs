use components::DepthOrderingTrait;
use components::model::Model;
use ecs::{Database, DatabaseTrait, DatabaseError};
use ecs::Entity;
use ecs::EventManagerTrait;
use event::Event;
use failure::Error;
use hierarchy::Hierarchy;
use std::any::Any;
use std::collections::VecDeque;
use std::hash::Hash;

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
    V: Clone + Default,
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
            let current_model = db.borrow(entity).ok()?;
            Some(parent_model * current_model)
        })?;
        Ok(())
    }

    fn insert_node(&mut self, entity: Entity) {
        self.scene_graph.insert(entity, Model::default())
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

    fn borrow<C: Any>(&self, entity: &Entity) -> Result<&C, DatabaseError> {
        self.database.borrow::<C>(entity)
    }

    fn borrow_mut<C: Any>(&mut self, entity: &Entity) -> Result<&mut C, DatabaseError> {
        self.database.borrow_mut::<C>(entity)
    }

    fn find<C: Any>(&self) -> Result<&C, DatabaseError> {
        self.database.find::<C>()
    }
}
