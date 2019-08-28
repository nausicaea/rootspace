//! Provides the resource manager.

use crate::{resource::Resource, components::Component, persistence::Persistence};
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use typename::TypeName;
use serde::{Serialize, Serializer, Deserialize, Deserializer};

#[derive(Debug)]
struct ResourceContainer {
    persistence: Persistence,
    inner: RefCell<Box<dyn Resource>>,
}

impl ResourceContainer {
    fn new<R>(resource: R, persistence: Persistence) -> Self
    where
        R: Resource,
    {
        ResourceContainer {
            persistence,
            inner: RefCell::new(Box::new(resource)),
        }
    }

    fn into_inner(self) -> RefCell<Box<dyn Resource>> {
        self.inner
    }
}

impl Deref for ResourceContainer {
    type Target = RefCell<Box<dyn Resource>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ResourceContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// A container that manages resources. Allows mutable borrows of multiple different resources at
/// the same time.
#[derive(Default, Debug)]
pub struct Resources(HashMap<TypeId, ResourceContainer>);

impl Resources {
    /// Empty the resource manager.
    pub fn clear(&mut self, persistence: Persistence) {
        self.0.retain(|_, rc| rc.persistence >= persistence)
    }

    /// Insert a new resource. Returns any previously present resource of the same type.
    pub fn insert<R>(&mut self, res: R, persistence: Persistence) -> Option<R>
    where
        R: Resource,
    {
        self.0
            .insert(TypeId::of::<R>(), ResourceContainer::new(res, persistence))
            .map(|r| {
                *r.into_inner()
                    .into_inner()
                    .downcast::<R>()
                    .expect("Could not downcast replaced resource to the specified type")
            })
    }

    /// Removes the resource of the specified type.
    pub fn remove<R>(&mut self) -> Option<R>
    where
        R: Resource,
    {
        self.0.remove(&TypeId::of::<R>()).map(|r| {
            *r.into_inner()
                .into_inner()
                .downcast::<R>()
                .expect("Could not downcast the removed resource to the specified type")
        })
    }

    /// Returns `true` if a resource of the specified type is present.
    pub fn has<R>(&mut self) -> bool
    where
        R: Resource,
    {
        self.0.contains_key(&TypeId::of::<R>())
    }

    /// Borrows the requested resource.
    pub fn borrow<R>(&self) -> Ref<R>
    where
        R: Resource + TypeName,
    {
        self.0
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
        self.0
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
        self.0
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

    /// Serialize the specified resource to the provided serializer.
    pub fn serialize<R, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        R: Resource + TypeName + Serialize,
        S: Serializer,
    {
        self.borrow::<R>().serialize(serializer)
    }

    /// Deserialize and insert the specified resource type from the provided deserializer.
    pub fn deserialize<'de, R, D>(&mut self, deserializer: D, persistence: Persistence) -> Result<(), D::Error>
    where
        R: Resource + Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let value: R = Deserialize::deserialize(deserializer)?;
        self.insert::<R>(value, persistence);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn serialize() {
        let mut resources = Resources::default();
        resources.insert(TestResource::default(), Persistence::Runtime);

        let mut writer: Vec<u8> = Vec::with_capacity(128);
        let mut s = serde_json::Serializer::new(&mut writer);
        assert!(resources.serialize::<TestResource, _>(&mut s).is_ok());
        assert_eq!(unsafe { String::from_utf8_unchecked(writer) }, "100");
    }

    #[test]
    fn deserialize() {
        let mut resources = Resources::default();

        let mut d = serde_json::Deserializer::from_slice(b"100");
        assert!(resources.deserialize::<TestResource, _>(&mut d, Persistence::Runtime).is_ok());
        assert!(d.end().is_ok());
        let _: Ref<TestResource> = resources.borrow();
    }
}
