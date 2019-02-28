//! Provides the resource manager.

use mopa::Any;
use crate::components::Component;
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt,
};

/// A resource is a data structure that is not coupled to a specific entity. Resources can be used
/// to provide "global" state to systems.
pub trait Resource: Any + fmt::Debug {}

mopafy!(Resource);

/// A container that manages resources. Allows mutable borrows of multiple resources at the same
/// time.
#[derive(Default)]
pub struct Resources(HashMap<TypeId, RefCell<Box<Resource>>>);

impl Resources {
    /// Empty the resource manager.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Insert a new resource. Returns any previously present resource of the same type.
    pub fn insert<R>(&mut self, res: R) -> Option<R>
    where
        R: Resource,
    {
        self.0
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(res)))
            .map(|r| *r.into_inner().downcast::<R>().expect("Could not downcast the resource"))
    }

    /// Removes the resource of the specified type.
    pub fn remove<R>(&mut self) -> Option<R>
    where
        R: Resource,
    {
        self.0
            .remove(&TypeId::of::<R>())
            .map(|r| *r.into_inner().downcast::<R>().expect("Could not downcast the resource"))
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
        R: Resource,
    {
        self.0
            .get(&TypeId::of::<R>())
            .map(|r| {
                Ref::map(r.borrow(), |i| {
                    i.downcast_ref::<R>().expect("Could not downcast the resource")
                })
            })
            .expect("Could not find the resource")
    }

    /// Mutably borrows the requested resource (with a runtime borrow check).
    pub fn borrow_mut<R>(&self) -> RefMut<R>
    where
        R: Resource,
    {
        self.0
            .get(&TypeId::of::<R>())
            .map(|r| {
                RefMut::map(r.borrow_mut(), |i| {
                    i.downcast_mut::<R>().expect("Could not downcast the resource")
                })
            })
            .expect("Could not find the resource")
    }

    /// Mutably borrows the requested resource (with a compile-time borrow check).
    pub fn get_mut<R>(&mut self) -> &mut R
    where
        R: Resource,
    {
        self.0
            .get_mut(&TypeId::of::<R>())
            .map(|r| {
                r.get_mut()
                    .downcast_mut::<R>()
                    .expect("Could not downcast the resource")
            })
            .expect("Could not find the resource")
    }

    /// Borrows the requested component storage (this is a convenience method to `borrow`).
    pub fn borrow_component<C>(&self) -> Ref<C::Storage>
    where
        C: Component,
    {
        self.borrow::<C::Storage>()
    }

    /// Mutably borrows the requested component storage (this is a convenience method to
    /// `borrow_mut`).
    pub fn borrow_mut_component<C>(&self) -> RefMut<C::Storage>
    where
        C: Component,
    {
        self.borrow_mut::<C::Storage>()
    }
}
