//! Provides the resource manager.
#![allow(non_snake_case)]

use std::{
    any::{type_name, TypeId},
    collections::{HashMap, HashSet},
};

use anyhow::Error;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{
    component::Component, registry::ResourceRegistry, resource::Resource, with_dependencies::WithDependencies,
};

macro_rules! impl_iter_ref {
    ($name:ident, $iter:ident, #reads: $($type:ident),+ $(,)?) => {
        impl_iter_ref!($name, $iter, #reads: $($type),+, #writes: );
    };

    ($name:ident, $iter:ident, #writes: $($type_mut:ident),+ $(,)?) => {
        impl_iter_ref!($name, $iter, #reads: , #writes: $($type_mut),+);
    };

    ($name:ident, $iter:ident, #reads: $($type:ident),*, #writes: $($type_mut:ident),* $(,)?) => {
        /// Creates a joined iterator over the specified group of components. In other words, only
        /// entities that have all the specified components will be iterated over.
        pub fn $name<$($type,)* $($type_mut,)*>(&self) -> $crate::storage::iterators::$iter<$($type::Storage,)* $($type_mut::Storage,)*>
        where
            $(
                $type: Component,
            )*
            $(
                $type_mut: Component,
            )*
        {
            $(
                let $type = self.read::<$type::Storage>();
            )*
            $(
                let $type_mut = self.write::<$type_mut::Storage>();
            )*

            $crate::storage::iterators::$iter::new($($type,)* $($type_mut,)*)
        }
    };
}

/// A container that manages resources. Allows mutable borrows of multiple different resources at
/// the same time.
#[derive(Default)]
pub struct Resources(HashMap<TypeId, RwLock<Box<dyn Resource>>>);

impl Resources {
    impl_iter_ref!(iter_r, RIterRef, #reads: C);

    impl_iter_ref!(iter_w, WIterRef, #writes: C);

    impl_iter_ref!(iter_rr, RRIterRef, #reads: C, D);

    impl_iter_ref!(iter_rw, RWIterRef, #reads: C, #writes: D);

    impl_iter_ref!(iter_ww, WWIterRef, #writes: C, D);

    impl_iter_ref!(iter_rrr, RRRIterRef, #reads: C, D, E);

    impl_iter_ref!(iter_rrw, RRWIterRef, #reads: C, D, #writes: E);

    impl_iter_ref!(iter_rww, RWWIterRef, #reads: C, #writes: D, E);

    impl_iter_ref!(iter_www, WWWIterRef, #writes: C, D, E);

    /// Create a new resources container with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Resources(HashMap::with_capacity(cap))
    }

    /// In a similar fashion to Resources::deserialize, the following method uses the types stored
    /// in the registry to initialize those resources that have a default, parameterless
    /// constructor.
    #[tracing::instrument(skip_all)]
    pub async fn with_dependencies<RR, D>(deps: &D) -> Result<Self, Error>
    where
        D: std::fmt::Debug,
        RR: ResourceRegistry + WithDependencies<D>,
    {
        fn recursive_insert<R: ResourceRegistry>(res: &mut Resources, reg: R) {
            if R::LEN == 0 {
                return;
            }

            let (head, tail) = reg.unzip();
            res.insert(head);
            recursive_insert(res, tail);
        }

        let rr = RR::with_deps(deps).await?;
        let mut res = Resources::with_capacity(RR::LEN);
        recursive_insert(&mut res, rr);

        Ok(res)
    }

    /// Clears the resources container.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Insert a new resource.
    pub fn insert<R>(&mut self, res: R)
    where
        R: Resource,
    {
        self.0.insert(TypeId::of::<R>(), RwLock::new(Box::new(res)));
    }

    /// Removes the resource of the specified type.
    pub fn remove<R>(&mut self)
    where
        R: Resource,
    {
        self.0.remove(&TypeId::of::<R>());
    }

    /// Returns `true` if a resource of the specified type is present.
    pub fn contains<R>(&self) -> bool
    where
        R: Resource,
    {
        self.0.contains_key(&TypeId::of::<R>())
    }

    /// Borrows the requested resource.
    pub fn read<R>(&self) -> MappedRwLockReadGuard<R>
    where
        R: Resource,
    {
        self.0
            .get(&TypeId::of::<R>())
            .ok_or(NoSuchTypeFound)
            .map(|r| {
                RwLockReadGuard::map(r.read(), |i| {
                    i.downcast_ref::<R>().unwrap_or_else(|| {
                        panic!("Could not downcast the requested resource to type {}", type_name::<R>())
                    })
                })
            })
            .unwrap_or_else(|e| panic!("Unable to acquire read access to resource {}: {}", type_name::<R>(), e))
    }

    /// Mutably borrows the requested resource (with a runtime borrow check).
    pub fn write<R>(&self) -> MappedRwLockWriteGuard<R>
    where
        R: Resource,
    {
        self.0
            .get(&TypeId::of::<R>())
            .ok_or(NoSuchTypeFound)
            .map(|r| {
                RwLockWriteGuard::map(r.write(), |i| {
                    i.downcast_mut::<R>().unwrap_or_else(|| {
                        panic!("Could not downcast the requested resource to type {}", type_name::<R>())
                    })
                })
            })
            .unwrap_or_else(|e| panic!("Unable to acquire write access to resource {}: {}", type_name::<R>(), e))
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
                    .unwrap_or_else(|| panic!("Could not downcast the requested resource to type {}", type_name::<R>()))
            })
            .unwrap_or_else(|| panic!("Could not find any resource of type {}", type_name::<R>()))
    }

    /// Borrows the requested component storage (this is a convenience method to `borrow`).
    pub fn read_components<C>(&self) -> MappedRwLockReadGuard<C::Storage>
    where
        C: Component,
    {
        self.read::<C::Storage>()
    }

    /// Mutably borrows the requested component storage (this is a convenience method to
    /// `borrow_mut`).
    pub fn write_components<C>(&self) -> MappedRwLockWriteGuard<C::Storage>
    where
        C: Component,
    {
        self.write::<C::Storage>()
    }

    /// Mutably borrows the requested component storage (with a compile-time borrow check).
    pub fn get_components_mut<C>(&mut self) -> &mut C::Storage
    where
        C: Component,
    {
        self.get_mut::<C::Storage>()
    }
}

impl PartialEq for Resources {
    fn eq(&self, rhs: &Resources) -> bool {
        if self.len() != rhs.len() {
            return false;
        }

        let lhs_k: HashSet<_> = self.0.keys().cloned().collect();
        let rhs_k: HashSet<_> = rhs.0.keys().cloned().collect();

        lhs_k == rhs_k
    }
}

impl std::fmt::Debug for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Resoources(#{})", self.0.len())
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Could not find any resource of the requested type")]
struct NoSuchTypeFound;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::Reg;

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceA(usize);

    impl Resource for TestResourceA {}

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceB(Vec<usize>);

    impl Resource for TestResourceB {}

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceC(String);

    impl Resource for TestResourceC {}

    #[test]
    fn test_registry() {
        type _TestRegistry = Reg![TestResourceA, TestResourceB, TestResourceC,];
    }

    #[test]
    fn default() {
        let _: Resources = Default::default();
    }

    #[test]
    fn insert() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA::default());
    }

    #[test]
    fn read() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA::default());

        let _: MappedRwLockReadGuard<TestResourceA> = resources.read();
    }
}
