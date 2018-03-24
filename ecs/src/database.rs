use std::any::{Any, TypeId};
use std::collections::HashMap;
use entity::Entity;

pub trait GroupTrait {
    fn borrow<T: Any>(&self) -> Option<&T>;
    fn borrow_mut<T: Any>(&mut self) -> Option<&mut T>;
}

pub trait DatabaseTrait: Default {}

pub struct Database<G> where G: GroupTrait {
    entities: HashMap<Entity, HashMap<TypeId, G>>,
}

impl<G> Default for Database<G> where G: GroupTrait {
    fn default() -> Self {
        Database {
            entities: Default::default(),
        }
    }
}

impl<G> Database<G> where G: GroupTrait {
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
    pub fn add_component<C>(&mut self, entity: &Entity, component: C) -> Result<(), Error> where C: Any + Into<G> {
        self.entities.get_mut(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| if !g.contains_key(&TypeId::of::<C>()) {
                g.insert(TypeId::of::<C>(), component.into());
                Ok(())
            } else {
                Err(Error::CannotOverwriteComponent)
            })
    }
    pub fn borrow<C>(&self, entity: &Entity) -> Result<&C, Error> where C: Any {
        self.entities.get(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                g.get(&TypeId::of::<C>())
                    .ok_or(Error::ComponentNotFound)
                    .map(|h| h.borrow().unwrap_or_else(|| unreachable!()))
            })
    }
    pub fn borrow_mut<C>(&mut self, entity: &Entity) -> Result<&mut C, Error> where C: Any {
        self.entities.get_mut(entity)
            .ok_or(Error::EntityNotFound)
            .and_then(|g| {
                g.get_mut(&TypeId::of::<C>())
                    .ok_or(Error::ComponentNotFound)
                    .map(|h| h.borrow_mut().unwrap_or_else(|| unreachable!()))
            })
    }
}

impl<G> DatabaseTrait for Database<G> where G: GroupTrait {
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

    #[derive(Debug, PartialEq)]
    enum TestGroup {
        A(TestTypeA),
        B(TestTypeB),
    }

    impl From<TestTypeA> for TestGroup {
        fn from(value: TestTypeA) -> Self {
            TestGroup::A(value)
        }
    }

    impl From<TestTypeB> for TestGroup {
        fn from(value: TestTypeB) -> Self {
            TestGroup::B(value)
        }
    }

    impl GroupTrait for TestGroup {
        fn borrow<T: Any>(&self) -> Option<&T> {
            use self::TestGroup::*;
            match *self {
                A(ref i) => if i.get_type_id() == TypeId::of::<T>() {
                    unsafe {
                        Some(&*(i as *const TestTypeA as *const T))
                    }
                } else {
                    None
                },
                B(ref i) => if i.get_type_id() == TypeId::of::<T>() {
                    unsafe {
                        Some(&*(i as *const TestTypeB as *const T))
                    }
                } else {
                    None
                },
            }
        }
        fn borrow_mut<T: Any>(&mut self) -> Option<&mut T> {
            use self::TestGroup::*;
            match *self {
                A(ref mut i) => if i.get_type_id() == TypeId::of::<T>() {
                    unsafe {
                        Some(&mut *(i as *mut TestTypeA as *mut T))
                    }
                } else {
                    None
                },
                B(ref mut i) => if i.get_type_id() == TypeId::of::<T>() {
                    unsafe {
                        Some(&mut *(i as *mut TestTypeB as *mut T))
                    }
                } else {
                    None
                },
            }
        }
    }

    #[test]
    fn create_entity() {
        let mut d: Database<TestGroup> = Database::default();
        assert!(d.entities.is_empty());
        let e = d.create_entity();
        assert!(d.entities.get(&e).is_some());
    }
    #[test]
    fn destroy_known_entity() {
        let mut d: Database<TestGroup> = Database::default();
        let e = d.create_entity();
        let r = d.destroy_entity(&e);
        assert!(r.is_ok());
        assert!(d.entities.is_empty());
    }
    #[test]
    fn destroy_unknown_entity() {
        let mut d: Database<TestGroup> = Database::default();
        let r = d.destroy_entity(&Default::default());
        assert!(r.is_err());
    }
    #[test]
    fn has_known_entity() {
        let mut d: Database<TestGroup> = Database::default();
        let e = d.create_entity();
        assert!(d.has_entity(&e));
    }
    #[test]
    fn has_unknown_entity() {
        let d: Database<TestGroup> = Database::default();
        let e = Entity::default();
        assert!(!d.has_entity(&e));
    }
    #[test]
    fn add_new_component() {
        let mut d: Database<TestGroup> = Database::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        let r = d.add_component(&e, c.clone());
        assert!(r.is_ok());
        assert_eq!(d.entities.get(&e).and_then(|g| g.get(&TypeId::of::<TestTypeA>())), Some(&TestGroup::A(c)));
        let c = TestTypeB::new(true);
        let r = d.add_component(&e, c.clone());
        assert!(r.is_ok());
        assert_eq!(d.entities.get(&e).and_then(|g| g.get(&TypeId::of::<TestTypeB>())), Some(&TestGroup::B(c)));
    }
    #[test]
    fn add_existing_component() {
        let mut d: Database<TestGroup> = Database::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        let r = d.add_component(&e, c.clone());
        assert!(r.is_ok());
        assert_eq!(d.entities.get(&e).and_then(|g| g.get(&TypeId::of::<TestTypeA>())), Some(&TestGroup::A(c.clone())));
        let r = d.add_component(&e, TestTypeA::new(2));
        assert!(r.is_err());
        assert_eq!(d.entities.get(&e).and_then(|g| g.get(&TypeId::of::<TestTypeA>())), Some(&TestGroup::A(c)));
    }
    #[test]
    fn add_component_unknown_entity() {
        let mut d: Database<TestGroup> = Database::default();
        let e = Entity::default();
        let c = TestTypeA::new(1);
        let r = d.add_component(&e, c.clone());
        assert!(r.is_err());
        assert!(d.entities.get(&e).is_none());
    }
    #[test]
    fn borrow_known_component() {
        let mut d: Database<TestGroup> = Database::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add_component(&e, c.clone()).unwrap();
        let r = d.borrow::<TestTypeA>(&e);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), &c);
    }
    #[test]
    fn borrow_unknown_component() {
        let mut d: Database<TestGroup> = Database::default();
        let e = d.create_entity();
        let r = d.borrow::<TestTypeA>(&e);
        assert!(r.is_err());
    }
    #[test]
    fn borrow_unknown_entity() {
        let d: Database<TestGroup> = Database::default();
        let e = Entity::default();
        let r = d.borrow::<TestTypeA>(&e);
        assert!(r.is_err());
    }
    #[test]
    fn borrow_mut() {
        let mut d: Database<TestGroup> = Database::default();
        let e = d.create_entity();
        let c = TestTypeA::new(1);
        d.add_component(&e, c.clone()).unwrap();
        let r = d.borrow_mut::<TestTypeA>(&e);
        assert!(r.is_ok());
        {
            let cb = r.unwrap();
            assert_eq!(cb, &c);
            cb.0 = 200;
        }
        assert_eq!(d.entities.get(&e).and_then(|g| g.get(&TypeId::of::<TestTypeA>())), Some(&TestGroup::A(TestTypeA(200))));
    }
}
