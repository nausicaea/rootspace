use std::any::{Any, TypeId};
use std::collections::HashMap;
use entity::Entity;

pub trait DatabaseTrait: Default {}

pub struct Database {
    entities: HashMap<Entity, HashMap<TypeId, Box<Any>>>,
}

impl Default for Database {
    fn default() -> Self {
        Database {
            entities: Default::default(),
        }
    }
}

impl Database {
    pub fn create_entity(&mut self) -> Entity {
        let e = Entity::default();
        self.entities.insert(e.clone(), Default::default());
        e
    }
    pub fn destroy_entity(&mut self, entity: &Entity) -> Result<(), Error> {
        self.entities.remove(entity)
            .map(|_| ())
            .ok_or(Error::EntityNotFound)
    }
    pub fn has_entity(&self, entity: &Entity) -> bool {
        self.entities.contains_key(entity)
    }
    pub fn entities(&self) -> usize {
        self.entities.len()
    }
    pub fn add<C>(&mut self, entity: &Entity, component: C) -> Result<(), Error> where C: Any {
        self.entities.get_mut(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| if !g.contains_key(&TypeId::of::<C>()) {
                g.insert(TypeId::of::<C>(), Box::new(component));
                Ok(())
            } else {
                Err(Error::CannotOverwriteComponent)
            })
    }
    pub fn remove<C>(&mut self, entity: &Entity) -> Result<C, Error> where C: Any {
        self.entities.get_mut(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                g.remove(&TypeId::of::<C>())
                    .ok_or(Error::ComponentNotFound)
                    .map(|h| *h.downcast().unwrap_or_else(|_| unreachable!()))
            })
    }
    pub fn has<C>(&self, entity: &Entity) -> bool where C: Any {
        self.entities.get(entity)
            .map(|g| g.contains_key(&TypeId::of::<C>()))
            .unwrap_or_default()
    }
    pub fn components(&self, entity: &Entity) -> usize {
        self.entities.get(entity)
            .map(|g| g.len())
            .unwrap_or_default()
    }
    pub fn borrow<C>(&self, entity: &Entity) -> Result<&C, Error> where C: Any {
        self.entities.get(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                g.get(&TypeId::of::<C>())
                    .ok_or(Error::ComponentNotFound)
                    .map(|h| h.downcast_ref().unwrap_or_else(|| unreachable!()))
            })
    }
    pub fn borrow_mut<C>(&mut self, entity: &Entity) -> Result<&mut C, Error> where C: Any {
        self.entities.get_mut(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                g.get_mut(&TypeId::of::<C>())
                    .ok_or(Error::ComponentNotFound)
                    .map(|h| h.downcast_mut().unwrap_or_else(|| unreachable!()))
            })
    }
}

impl DatabaseTrait for Database {
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "No such entity")]
    EntityNotFound,
    #[fail(display = "There is already a component of that type")]
    CannotOverwriteComponent,
    #[fail(display = "No such component")]
    ComponentNotFound,
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
    fn create_entity() {
        let mut d: Database = Default::default();
        assert_eq!(d.entities(), 0);
        let e = d.create_entity();
        assert_eq!(d.entities(), 1);
        assert_eq!(d.components(&e), 0);
    }

    #[test]
    fn destroy_known_entity() {
        let mut d: Database = Default::default();
        assert_eq!(d.entities(), 0);
        let e = d.create_entity();
        assert_eq!(d.entities(), 1);
        let r = d.destroy_entity(&e);
        assert_ok!(r);
        assert_eq!(d.entities(), 0);
    }

    #[test]
    fn destroy_unknown_entity() {
        let mut d: Database = Default::default();
        assert_eq!(d.entities(), 0);
        let _e = d.create_entity();
        assert_eq!(d.entities(), 1);
        let r = d.destroy_entity(&Default::default());
        assert_err!(r);
        assert_eq!(d.entities(), 1);
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
    fn add_new_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        assert_ok!(d.add(&e, c.clone()));
        assert_eq!(d.components(&e), 1);
        let c = TestTypeB::new(true);
        assert_ok!(d.add(&e, c.clone()));
        assert_eq!(d.components(&e), 2);
    }

    #[test]
    fn add_existing_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        assert_ok!(d.add(&e, c.clone()));
        assert_eq!(d.components(&e), 1);
        assert_err!(d.add(&e, TestTypeA::new(2)));
        assert_eq!(d.components(&e), 1);
        assert_eq!(d.borrow::<TestTypeA>(&e).unwrap(), &c);
    }

    #[test]
    fn add_component_unknown_entity() {
        let mut d: Database = Default::default();
        let e: Entity = Default::default();
        let c = TestTypeA::new(1);
        assert_err!(d.add(&e, c.clone()));
        assert_eq!(d.components(&e), 0);
    }

    #[test]
    fn remove_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(&e, c.clone()).unwrap();
        assert_eq!(d.components(&e), 1);
        let r = d.remove::<TestTypeA>(&e);
        assert_ok!(r);
        assert_eq!(r.unwrap(), c);
        assert_eq!(d.components(&e), 0);
    }

    #[test]
    fn remove_unknown_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert_err!(d.remove::<TestTypeA>(&e));
    }

    #[test]
    fn remove_component_unknown_entity() {
        let mut d: Database = Default::default();
        let e: Entity = Default::default();
        assert_err!(d.remove::<TestTypeA>(&e));
    }

    #[test]
    fn has_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(&e, c.clone()).unwrap();
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
        d.add(&e, c.clone()).unwrap();
        let r = d.borrow::<TestTypeA>(&e);
        assert_ok!(r);
        assert_eq!(r.unwrap(), &c);
    }

    #[test]
    fn borrow_unknown_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert_err!(d.borrow::<TestTypeA>(&e));
    }

    #[test]
    fn borrow_component_unknown_entity() {
        let d: Database = Default::default();
        let e: Entity = Default::default();
        assert_err!(d.borrow::<TestTypeA>(&e));
    }

    #[test]
    fn borrow_mut_known_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add(&e, c.clone()).unwrap();
        {
            let r = d.borrow_mut::<TestTypeA>(&e);
            assert_ok!(r);
            let cb = r.unwrap();
            assert_eq!(cb, &c);
            cb.0 = 200;
        }
        {
            let r = d.borrow::<TestTypeA>(&e);
            assert_ok!(r);
            let cb = r.unwrap();
            assert_eq!(cb, &TestTypeA(200));
        }
    }

    #[test]
    fn borrow_mut_unknown_component() {
        let mut d: Database = Default::default();
        let e = d.create_entity();
        assert_err!(d.borrow_mut::<TestTypeA>(&e));
    }

    #[test]
    fn borrow_mut_unknown_entity() {
        let mut d: Database = Default::default();
        let e: Entity = Default::default();
        assert_err!(d.borrow_mut::<TestTypeA>(&e));
    }
}
