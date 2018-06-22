use database::{Database, DatabaseTrait, Error as DatabaseError};
use entity::Entity;
use event::{EventTrait, EventManagerTrait};
use failure::Error;
use std::any::Any;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct MockCtx<E>
where
    E: EventTrait,
{
    pub events: VecDeque<E>,
    pub handle_events_calls: usize,
    pub database: Database,
}

impl<E> Default for MockCtx<E>
where
    E: EventTrait,
{
    fn default() -> Self {
        MockCtx {
            events: Default::default(),
            handle_events_calls: 0,
            database: Default::default(),
        }
    }
}

impl<E> EventManagerTrait<E> for MockCtx<E>
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
        self.handle_events_calls += 1;

        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        for event in tmp {
            handler(self, &event)?;
        }

        Ok(true)
    }
}

impl<E> DatabaseTrait for MockCtx<E>
where
    E: EventTrait,
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
