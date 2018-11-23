use components::model::Model;
use components::ui_model::UiModel;
use ecs::{Database, DatabaseError, DatabaseTrait, Entity, EventManagerTrait, EventTrait};
use failure::Error;
use hierarchy::Hierarchy;
use std::{
    any::Any,
    collections::{HashSet, VecDeque},
    fmt,
};

pub trait SceneGraphTrait {
    fn update_graph(&mut self) -> Result<(), Error>;
    fn insert_world_node(&mut self, entity: Entity);
    fn insert_ui_node(&mut self, entity: Entity);
    fn get_world_node(&self, entity: &Entity) -> Option<&Model>;
    fn get_ui_node(&self, entity: &Entity) -> Option<&UiModel>;
    fn get_world_nodes(&self) -> Vec<(&Entity, &Model)>;
    fn get_ui_nodes(&self) -> Vec<(&Entity, &UiModel)>;
}

pub struct Context<E> {
    events: VecDeque<E>,
    world_graph: Hierarchy<Entity, Model>,
    ui_graph: Hierarchy<Entity, UiModel>,
    database: Database,
}

impl<E> Context<E> {
    pub fn clear(&mut self) {
        self.events.clear();
        self.world_graph.clear();
        self.ui_graph.clear();
        self.database.clear();
    }
}

impl<E> fmt::Debug for Context<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context {{ ... }}")
    }
}

impl<E> Default for Context<E> {
    fn default() -> Self {
        Context {
            events: VecDeque::default(),
            world_graph: Hierarchy::default(),
            ui_graph: Hierarchy::default(),
            database: Database::default(),
        }
    }
}

impl<E> EventManagerTrait<E> for Context<E>
where
    E: EventTrait,
{
    fn dispatch_later(&mut self, event: E) {
        self.events.push_back(event)
    }

    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
    where
        F: FnMut(&mut Self, &E) -> Result<bool, Error>,
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

impl<E> SceneGraphTrait for Context<E> {
    fn update_graph(&mut self) -> Result<(), Error> {
        let db = &self.database;
        self.world_graph.update(&|entity, _, parent_model| {
            let current_model: &Model = db.get(entity).ok()?;
            Some(parent_model * current_model)
        })?;
        self.ui_graph.update(&|entity, _, parent_model| {
            let current_model: &UiModel = db.get(entity).ok()?;
            Some(parent_model * current_model)
        })?;

        Ok(())
    }

    fn insert_world_node(&mut self, entity: Entity) {
        self.world_graph.insert(entity, Default::default())
    }

    fn insert_ui_node(&mut self, entity: Entity) {
        self.ui_graph.insert(entity, Default::default())
    }

    fn get_world_node(&self, entity: &Entity) -> Option<&Model> {
        self.world_graph
            .iter()
            .filter(|&(k, _)| k == entity)
            .map(|(_, v)| v)
            .last()
    }

    fn get_ui_node(&self, entity: &Entity) -> Option<&UiModel> {
        self.ui_graph
            .iter()
            .filter(|&(k, _)| k == entity)
            .map(|(_, v)| v)
            .last()
    }

    fn get_world_nodes(&self) -> Vec<(&Entity, &Model)> {
        self.world_graph.iter().collect()
    }

    fn get_ui_nodes(&self) -> Vec<(&Entity, &UiModel)> {
        self.ui_graph.iter().collect()
    }
}

impl<E> DatabaseTrait for Context<E> {
    fn clear(&mut self) {
        self.database.clear()
    }

    fn create_entity(&mut self) -> Entity {
        self.database.create_entity()
    }

    fn destroy_entity(&mut self, entity: &Entity) -> Result<(), DatabaseError> {
        self.database.destroy_entity(entity)
    }

    fn has_entity(&self, entity: &Entity) -> bool {
        self.database.has_entity(entity)
    }

    fn num_entities(&self) -> usize {
        self.database.num_entities()
    }

    fn entities(&self) -> HashSet<&Entity> {
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

    fn num_components(&self, entity: &Entity) -> usize {
        self.database.num_components(entity)
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

    fn find_mut<C: Any>(&mut self) -> Result<&mut C, DatabaseError> {
        self.database.find_mut::<C>()
    }
}
