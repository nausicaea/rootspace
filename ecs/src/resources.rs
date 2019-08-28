//! Provides the resource manager.

use crate::{resource::Resource, components::Component, persistence::Persistence};
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};
use typename::TypeName;
// use serde::{ser::{SerializeStruct, Serialize, Serializer, SerializeMap}, de::{self, Deserializer, Visitor, MapAccess}, Deserialize};

/// A container that manages resources. Allows mutable borrows of multiple different resources at
/// the same time.
#[derive(Default, Debug)]
pub struct Resources {
    resources: HashMap<TypeId, RefCell<Box<dyn Resource>>>,
    persistences: HashMap<TypeId, Persistence>,
}

impl Resources {
    /// Create a new, empty resources container.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new resources container with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Resources {
            resources: HashMap::with_capacity(cap),
            persistences: HashMap::with_capacity(cap),
        }
    }

    /// Empty the resource manager.
    pub fn clear(&mut self, persistence: Persistence) {
        let persistences = &self.persistences;
        self.resources.retain(|id, _| persistences[id] >= persistence)
    }

    /// Insert a new resource.
    pub fn insert<R>(&mut self, res: R, persistence: Persistence)
        where
        R: Resource,
        {
            self.resources.insert(TypeId::of::<R>(), RefCell::new(Box::new(res)));
            self.persistences.insert(TypeId::of::<R>(), persistence);
        }

    /// Removes the resource of the specified type.
    pub fn remove<R>(&mut self)
        where
        R: Resource,
        {
            self.resources.remove(&TypeId::of::<R>());
            self.persistences.remove(&TypeId::of::<R>());
        }

    /// Returns `true` if a resource of the specified type is present.
    pub fn has<R>(&self) -> bool
        where
        R: Resource,
        {
            self.resources.contains_key(&TypeId::of::<R>())
        }

    /// Returns the persistence of the specified resource type.
    pub fn persistence_of<R>(&self) -> Persistence
    where
        R: Resource + TypeName,
    {
        *self.persistences.get(&TypeId::of::<R>())
            .expect(&format!("Could not find any resource of type {}", R::type_name()))
    }

    /// Borrows the requested resource.
    pub fn borrow<R>(&self) -> Ref<R>
        where
        R: Resource + TypeName,
        {
            self.resources
                .get(&TypeId::of::<R>())
                .map(|r| {
                    Ref::map(r.borrow(), |i| {
                        i.downcast_ref::<R>().expect(&format!(
                                "Could not downcast the requested resource to type {}",
                                R::type_name()
                        ))
                    })
                })
            .expect(&format!("Could not find any resource of type {}", R::type_name()))
        }

    /// Mutably borrows the requested resource (with a runtime borrow check).
    pub fn borrow_mut<R>(&self) -> RefMut<R>
        where
        R: Resource + TypeName,
        {
            self.resources
                .get(&TypeId::of::<R>())
                .map(|r| {
                    RefMut::map(r.borrow_mut(), |i| {
                        i.downcast_mut::<R>().expect(&format!(
                                "Could not downcast the requested resource to type {}",
                                R::type_name()
                        ))
                    })
                })
            .expect(&format!("Could not find any resource of type {}", R::type_name()))
        }

    /// Mutably borrows the requested resource (with a compile-time borrow check).
    pub fn get_mut<R>(&mut self) -> &mut R
        where
        R: Resource + TypeName,
        {
            self.resources
                .get_mut(&TypeId::of::<R>())
                .map(|r| {
                    r.get_mut().downcast_mut::<R>().expect(&format!(
                            "Could not downcast the requested resource to type {}",
                            R::type_name()
                    ))
                })
            .expect(&format!("Could not find any resource of type {}", R::type_name()))
        }

    /// Borrows the requested component storage (this is a convenience method to `borrow`).
    pub fn borrow_component<C>(&self) -> Ref<C::Storage>
        where
        C: Component + TypeName,
        C::Storage: TypeName,
        {
            self.borrow::<C::Storage>()
        }

    /// Mutably borrows the requested component storage (this is a convenience method to
    /// `borrow_mut`).
    pub fn borrow_mut_component<C>(&self) -> RefMut<C::Storage>
        where
        C: Component + TypeName,
        C::Storage: TypeName,
        {
            self.borrow_mut::<C::Storage>()
        }
}

// pub trait SerDeTrait<T> {
//     fn serialize<S: Serializer>(&self, serializer: S) -> Result<(), S::Error>;
//     fn deserialize<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error>;
// }
//
// impl SerDeTrait<(R0,)> for Resources
// where
//     R0: Resource + TypeName + Serialize,
// {
//     fn serialize<S: Serializer>(&self, serializer: S) -> Result<(), S::Error> {
//         let mut state = serializer.serialize_struct("Resources", 2)?;
//         let r0 = self.borrow::<R0>();
//         let r0_per = self.persistence_of::<R0>();
//         map.serialize_entry(TypeId::of::<R0>(), Container::new(r0_per, r0
//         map.end()?;
//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, TypeName, Serialize, Deserialize)]
    struct TestResource(usize);

    impl Default for TestResource {
        fn default() -> Self {
            TestResource(100)
        }
    }

    impl Resource for TestResource {}

    #[test]
    fn persistence() {
        assert!(Persistence::None < Persistence::Runtime);
    }

    #[test]
    fn default() {
        let _: Resources = Default::default();
    }

    #[test]
    fn insert() {
        let mut resources = Resources::default();
        resources.insert(TestResource::default(), Persistence::Runtime);
    }

    #[test]
    fn borrow() {
        let mut resources = Resources::default();
        resources.insert(TestResource::default(), Persistence::Runtime);

        let _: Ref<TestResource> = resources.borrow();
    }

    // #[test]
    // fn serialize() {
    //     let mut resources = Resources::default();
    //     resources.insert(TestResource::default(), Persistence::Runtime);

    //     let mut writer: Vec<u8> = Vec::with_capacity(128);
    //     let mut s = serde_json::Serializer::new(&mut writer);
    //     assert!(resources.serialize::<TestResource, _>(&mut s).is_ok());
    //     assert_eq!(unsafe { String::from_utf8_unchecked(writer) }, "{\"persistence\":\"Runtime\",\"resource\":100}");
    // }

    // #[test]
    // fn deserialize() {
    //     let mut resources = Resources::default();

    //     let mut d = serde_json::Deserializer::from_slice(b"{\"persistence\":\"Runtime\",\"resource\":100}");
    //     resources.deserialize::<TestResource, _>(&mut d).unwrap();
    //     assert!(d.end().is_ok());
    //     let _: Ref<TestResource> = resources.borrow();
    // }
}
