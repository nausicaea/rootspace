//! Provides the `Database` container, which creates a relationship between `Entity` and its
//! components (arbitrary data).

use crate::entity::Entity;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

/// This trait provides access to data stored in a entity-components relationship database.
pub trait DatabaseTrait: Default {
    /// Removes all stored data and resets the database.
    fn clear(&mut self);
    /// Creates a new entity (including storage space) and returns the identifier.
    fn create_entity(&mut self) -> Entity;
    /// Destroys an entity and associated data.
    fn destroy_entity(&mut self, entity: &Entity) -> Result<(), Error>;
    /// Returns `true` if the specified entity is known.
    fn has_entity(&self, entity: &Entity) -> bool;
    /// Returns the number of known entities.
    fn num_entities(&self) -> usize;
    /// Returns a vector of the known entities.
    fn entities(&self) -> HashSet<&Entity>;
    /// Adds a component (e.g. an arbitrary type) to the specified entity.
    fn add<C: Any>(&mut self, entity: Entity, component: C) -> Result<(), Error>;
    /// Removes a component of the specified type from an entity.
    fn remove<C: Any>(&mut self, entity: &Entity) -> Result<C, Error>;
    /// Returns `true` if the entity contains a component of the specified type.
    fn has<C: Any>(&self, entity: &Entity) -> bool;
    /// Returns the number of components for the specified entity.
    fn num_components(&self, entity: &Entity) -> usize;
    /// Retrieves a reference to the component of the specified type on an entity.
    fn get<C: Any>(&self, entity: &Entity) -> Result<&C, Error>;
    /// Retrieves a mutable reference to the component of the specified type on an entity.
    fn get_mut<C: Any>(&mut self, entity: &Entity) -> Result<&mut C, Error>;
    /// Retrieves a reference to a *single* component of the specified type, regardless of the
    /// containing entity.
    fn find<C: Any>(&self) -> Result<&C, Error>;
    /// Retrieves a mutable reference to a *single* component of the specified type, regardless of
    /// the containing entity.
    fn find_mut<C: Any>(&mut self) -> Result<&mut C, Error>;
}

/// The `Database` stores entities and their data as a nested hash map of boxed trait objects
/// indexed by entities and component type id.
#[derive(Debug)]
// #[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Database {
    entities: HashMap<Entity, HashMap<TypeId, Box<Any>>>,
}

impl Default for Database {
    fn default() -> Self {
        Database {
            entities: HashMap::default(),
        }
    }
}

impl DatabaseTrait for Database {
    fn clear(&mut self) {
        self.entities.clear()
    }

    fn create_entity(&mut self) -> Entity {
        let e = Entity::default();
        self.entities.insert(e, HashMap::default());
        e
    }

    fn destroy_entity(&mut self, entity: &Entity) -> Result<(), Error> {
        self.entities.remove(entity).map(|_| ()).ok_or(Error::EntityNotFound)
    }

    fn has_entity(&self, entity: &Entity) -> bool {
        self.entities.contains_key(entity)
    }

    fn num_entities(&self) -> usize {
        self.entities.len()
    }

    fn entities(&self) -> HashSet<&Entity> {
        self.entities.keys().collect()
    }

    fn add<C: Any>(&mut self, entity: Entity, component: C) -> Result<(), Error> {
        self.entities
            .get_mut(&entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                if !g.contains_key(&TypeId::of::<C>()) {
                    g.insert(TypeId::of::<C>(), Box::new(component));
                    Ok(())
                } else {
                    Err(Error::CannotOverwriteComponent)
                }
            })
    }

    fn remove<C: Any>(&mut self, entity: &Entity) -> Result<C, Error> {
        self.entities
            .get_mut(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                g.remove(&TypeId::of::<C>())
                    .ok_or(Error::ComponentNotFound)
                    .map(|h| *h.downcast().unwrap_or_else(|_| unreachable!()))
            })
    }

    fn has<C: Any>(&self, entity: &Entity) -> bool {
        self.entities
            .get(entity)
            .map(|g| g.contains_key(&TypeId::of::<C>()))
            .unwrap_or_default()
    }

    fn num_components(&self, entity: &Entity) -> usize {
        self.entities.get(entity).map(|g| g.len()).unwrap_or_default()
    }

    fn get<C: Any>(&self, entity: &Entity) -> Result<&C, Error> {
        self.entities.get(entity).ok_or(Error::EntityNotFound).and_then(|g| {
            g.get(&TypeId::of::<C>())
                .ok_or(Error::ComponentNotFound)
                .map(|h| h.downcast_ref().unwrap_or_else(|| unreachable!()))
        })
    }

    fn get_mut<C: Any>(&mut self, entity: &Entity) -> Result<&mut C, Error> {
        self.entities
            .get_mut(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                g.get_mut(&TypeId::of::<C>())
                    .ok_or(Error::ComponentNotFound)
                    .map(|h| h.downcast_mut().unwrap_or_else(|| unreachable!()))
            })
    }

    fn find<C: Any>(&self) -> Result<&C, Error> {
        let candidates = self
            .entities
            .values()
            .filter(|g| g.contains_key(&TypeId::of::<C>()))
            .map(|g| {
                g.get(&TypeId::of::<C>())
                    .unwrap()
                    .downcast_ref()
                    .unwrap_or_else(|| unreachable!())
            }).collect::<Vec<&C>>();

        if candidates.len() == 1 {
            Ok(candidates.into_iter().last().unwrap())
        } else if candidates.is_empty() {
            Err(Error::ComponentNotFound)
        } else {
            Err(Error::MultipleComponentsFound)
        }
    }

    fn find_mut<C: Any>(&mut self) -> Result<&mut C, Error> {
        let candidates = self
            .entities
            .values_mut()
            .filter(|g| g.contains_key(&TypeId::of::<C>()))
            .map(|g| {
                g.get_mut(&TypeId::of::<C>())
                    .unwrap()
                    .downcast_mut()
                    .unwrap_or_else(|| unreachable!())
            }).collect::<Vec<&mut C>>();

        if candidates.len() == 1 {
            Ok(candidates.into_iter().last().unwrap())
        } else if candidates.is_empty() {
            Err(Error::ComponentNotFound)
        } else {
            Err(Error::MultipleComponentsFound)
        }
    }
}

/// Describes errors that may occur when operating on databases.
#[derive(Debug, Fail)]
pub enum Error {
    /// The specified entity is not known to the database.
    #[fail(display = "No such entity")]
    EntityNotFound,
    /// Occurs when trying to add a component to an entity that already has one of the same type.
    #[fail(display = "There is already a component of that type")]
    CannotOverwriteComponent,
    /// Occurs if the specified component was not found.
    #[fail(display = "No such component")]
    ComponentNotFound,
    /// Occurs if the specified component was found more than once.
    #[fail(display = "Multiple components were found where one was expected")]
    MultipleComponentsFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestTypeA(u32);

    impl TestTypeA {
        pub fn new(d: u32) -> Self {
            TestTypeA(d)
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestTypeB(bool);

    impl TestTypeB {
        pub fn new(d: bool) -> Self {
            TestTypeB(d)
        }
    }

    #[test]
    fn clear() {
        let mut d = Database::default();
        let _e = d.create_entity();
        assert_eq!(d.num_entities(), 1);
        d.clear();
        assert_eq!(d.num_entities(), 0);
    }

    #[test]
    fn create_entity() {
        let mut d: Database = Default::default();
        assert_eq!(d.num_entities(), 0);
        let e = d.create_entity();
        assert_eq!(d.num_entities(), 1);
        assert_eq!(d.num_components(&e), 0);
    }

    #[test]
    fn destroy_known_entity() {
        let mut d: Database = Default::default();
        assert_eq!(d.num_entities(), 0);
        let e = d.create_entity();
        assert_eq!(d.num_entities(), 1);
        let r = d.destroy_entity(&e);
        assert!(r.is_ok());
        assert_eq!(d.num_entities(), 0);
    }

    #[test]
    fn destroy_unknown_entity() {
        let mut d: Database = Default::default();
        assert_eq!(d.num_entities(), 0);
        let _e = d.create_entity();
        assert_eq!(d.num_entities(), 1);
        let r = d.destroy_entity(&Default::default());
        assert!(r.is_err());
        assert_eq!(d.num_entities(), 1);
    }

    #[test]
    fn has_known_entity() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert!(d.has_entity(&e));
    }

    #[test]
    fn has_unknown_entity() {
        let d: Database = Default::default();
        let e: Entity = Default::default();
        assert!(!d.has_entity(&e));
    }

    #[test]
    fn retrieve_entities() {
        let mut d: Database = Default::default();
        let a = d.create_entity();
        let b = d.create_entity();
        let expected = [a, b];
        assert_eq!(d.entities(), expected.iter().collect());
    }

    #[test]
    fn add_new_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        assert!(d.add(e.clone(), c.clone()).is_ok());
        assert_eq!(d.num_components(&e), 1);
        let c = TestTypeB::new(true);
        assert!(d.add(e.clone(), c.clone()).is_ok());
        assert_eq!(d.num_components(&e), 2);
    }

    #[test]
    fn add_existing_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        assert!(d.add(e.clone(), c.clone()).is_ok());
        assert_eq!(d.num_components(&e), 1);
        assert!(d.add(e.clone(), TestTypeA::new(2)).is_err());
        assert_eq!(d.num_components(&e), 1);
        assert_eq!(d.get::<TestTypeA>(&e).unwrap(), &c);
    }

    #[test]
    fn add_component_unknown_entity() {
        let mut d: Database = Default::default();
        let e: Entity = Default::default();
        let c = TestTypeA::new(1);
        assert!(d.add(e.clone(), c.clone()).is_err());
        assert_eq!(d.num_components(&e), 0);
    }

    #[test]
    fn remove_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(e.clone(), c.clone()).unwrap();
        assert_eq!(d.num_components(&e), 1);
        let r = d.remove::<TestTypeA>(&e);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), c);
        assert_eq!(d.num_components(&e), 0);
    }

    #[test]
    fn remove_unknown_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert!(d.remove::<TestTypeA>(&e).is_err());
    }

    #[test]
    fn remove_component_unknown_entity() {
        let mut d: Database = Default::default();
        let e: Entity = Default::default();
        assert!(d.remove::<TestTypeA>(&e).is_err());
    }

    #[test]
    fn has_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(e.clone(), c.clone()).unwrap();
        assert!(d.has::<TestTypeA>(&e));
    }

    #[test]
    fn has_unknown_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert!(!d.has::<TestTypeA>(&e));
    }

    #[test]
    fn has_component_unknown_entity() {
        let d: Database = Default::default();
        let e: Entity = Default::default();
        assert!(!d.has::<TestTypeA>(&e));
    }

    #[test]
    fn borrow_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(e.clone(), c.clone()).unwrap();
        let r = d.get::<TestTypeA>(&e);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), &c);
    }

    #[test]
    fn borrow_unknown_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert!(d.get::<TestTypeA>(&e).is_err());
    }

    #[test]
    fn borrow_component_unknown_entity() {
        let d: Database = Default::default();
        let e: Entity = Default::default();
        assert!(d.get::<TestTypeA>(&e).is_err());
    }

    #[test]
    fn borrow_mut_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(e.clone(), c.clone()).unwrap();
        {
            let r = d.get_mut::<TestTypeA>(&e);
            assert!(r.is_ok());
            let cb = r.unwrap();
            assert_eq!(cb, &c);
            cb.0 = 200;
        }
        {
            let r = d.get::<TestTypeA>(&e);
            assert!(r.is_ok());
            let cb = r.unwrap();
            assert_eq!(cb, &TestTypeA(200));
        }
    }

    #[test]
    fn borrow_mut_unknown_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert!(d.get_mut::<TestTypeA>(&e).is_err());
    }

    #[test]
    fn borrow_mut_unknown_entity() {
        let mut d: Database = Default::default();
        let e: Entity = Default::default();
        assert!(d.get_mut::<TestTypeA>(&e).is_err());
    }

    #[test]
    fn find_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(e.clone(), c.clone()).unwrap();
        let r = d.find::<TestTypeA>();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), &c);
    }

    #[test]
    fn find_unknown_component() {
        let mut d: Database = Default::default();
        let _e = d.create_entity();
        assert!(d.find::<TestTypeA>().is_err());
    }

    #[test]
    fn find_duplicate_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let x = TestTypeA::new(1);
        d.add(e.clone(), x.clone()).unwrap();
        let f = d.create_entity();
        let y = TestTypeA::new(2);
        d.add(f.clone(), y.clone()).unwrap();
        assert!(d.find::<TestTypeA>().is_err());
    }
}
