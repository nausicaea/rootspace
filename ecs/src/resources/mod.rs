//! Provides the resource manager.
#![allow(non_snake_case)]

use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
};

use anyhow::Error;
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use try_default::TryDefault;

use self::typed_resources::TypedResources;
use crate::{component::Component, registry::ResourceRegistry, resource::Resource, short_type_name::short_type_name};

mod recursors;
pub(crate) mod typed_resources;

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
                let $type = self.borrow::<$type::Storage>();
            )*
            $(
                let $type_mut = self.borrow_mut::<$type_mut::Storage>();
            )*

            $crate::storage::iterators::$iter::new($($type,)* $($type_mut,)*)
        }
    };
}

/// A container that manages resources. Allows mutable borrows of multiple different resources at
/// the same time.
#[derive(Default)]
pub struct Resources(HashMap<TypeId, RefCell<Box<dyn Resource>>>);

impl std::fmt::Debug for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Resoources(#{})", self.0.len())
    }
}

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
    pub fn with_registry<RR>() -> Result<Self, Error>
    where
        RR: ResourceRegistry,
    {
        let helper = TypedResources::<RR>::try_default()?;

        Ok(Resources::from(helper))
    }

    /// Deserialize [`Resources`](Self) with the supplied
    /// [`ResourceRegistry`](crate::registry::ResourceRegistry). Here, the registry determines the types that are
    pub fn deserialize_with<'de, RR, D>(deserializer: D) -> Result<Self, D::Error>
    where
        RR: ResourceRegistry,
        D: Deserializer<'de>,
    {
        let helper = TypedResources::<RR>::deserialize(deserializer)?;
        Ok(Resources::from(helper))
    }

    /// Serialize [`Resources`](Self) with the supplied
    /// [`ResourceRegistry`](crate::registry::ResourceRegistry).
    pub fn serialize_with<RR, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        RR: ResourceRegistry,
        S: Serializer,
    {
        let status = TypedResources::<RR>::from(self).serialize(serializer)?;
        Ok(status)
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
        self.0.insert(TypeId::of::<R>(), RefCell::new(Box::new(res)));
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
    pub fn borrow<R>(&self) -> Ref<R>
    where
        R: Resource,
    {
        self.0
            .get(&TypeId::of::<R>())
            .map(|r| {
                Ref::map(r.borrow(), |i| {
                    i.downcast_ref::<R>().unwrap_or_else(|| {
                        panic!(
                            "Could not downcast the requested resource to type {}",
                            short_type_name::<R>()
                        )
                    })
                })
            })
            .unwrap_or_else(|| panic!("Could not find any resource of type {}", short_type_name::<R>()))
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
                    i.downcast_mut::<R>().unwrap_or_else(|| {
                        panic!(
                            "Could not downcast the requested resource to type {}",
                            short_type_name::<R>()
                        )
                    })
                })
            })
            .unwrap_or_else(|| panic!("Could not find any resource of type {}", short_type_name::<R>()))
    }

    /// Mutably borrows the requested resource (with a compile-time borrow check).
    pub fn get_mut<R>(&mut self) -> &mut R
    where
        R: Resource,
    {
        self.0
            .get_mut(&TypeId::of::<R>())
            .map(|r| {
                r.get_mut().downcast_mut::<R>().unwrap_or_else(|| {
                    panic!(
                        "Could not downcast the requested resource to type {}",
                        short_type_name::<R>()
                    )
                })
            })
            .unwrap_or_else(|| panic!("Could not find any resource of type {}", short_type_name::<R>()))
    }

    /// Borrows the requested component storage (this is a convenience method to `borrow`).
    pub fn borrow_components<C>(&self) -> Ref<C::Storage>
    where
        C: Component,
    {
        self.borrow::<C::Storage>()
    }

    /// Mutably borrows the requested component storage (this is a convenience method to
    /// `borrow_mut`).
    pub fn borrow_components_mut<C>(&self) -> RefMut<C::Storage>
    where
        C: Component,
    {
        self.borrow_mut::<C::Storage>()
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

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json;

    use super::*;
    use crate::{world::event::WorldEvent, Entities, EventQueue, Reg, SerializationName};

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceA(usize);

    impl Resource for TestResourceA {}

    impl SerializationName for TestResourceA {}

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceB(Vec<usize>);

    impl Resource for TestResourceB {}

    impl SerializationName for TestResourceB {}

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceC(String);

    impl Resource for TestResourceC {}

    impl SerializationName for TestResourceC {}

    type TestRegistry = Reg![TestResourceA, TestResourceB, TestResourceC,];

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
    fn borrow() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA::default());

        let _: Ref<TestResourceA> = resources.borrow();
    }

    #[test]
    fn serialize_with() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA(25));
        resources.insert(TestResourceB(Vec::new()));
        resources.insert(TestResourceC(String::from("Bye")));

        let mut writer: Vec<u8> = Vec::with_capacity(128);
        let mut s = serde_json::Serializer::new(&mut writer);
        assert!(resources.serialize_with::<TestRegistry, _>(&mut s).is_ok());
        assert_eq!(
            unsafe { String::from_utf8_unchecked(writer) },
            "{\"TestResourceA\":25,\"TestResourceB\":[],\"TestResourceC\":\"Bye\"}"
        );
    }

    #[test]
    fn deserialize_with() {
        let mut d = serde_json::Deserializer::from_slice(
            b"{\"TestResourceA\":25,\"TestResourceB\":[0,1],\"TestResourceC\":\"Bye\"}",
        );
        let resources = Resources::deserialize_with::<TestRegistry, _>(&mut d).unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(25));
        assert_eq!(*resources.borrow::<TestResourceB>(), TestResourceB(vec![0, 1]));
        assert_eq!(*resources.borrow::<TestResourceC>(), TestResourceC(String::from("Bye")));
    }

    #[test]
    fn deserialize_with_complex() {
        let mut d =
            serde_json::Deserializer::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ok.json")));
        let r = Resources::deserialize_with::<Reg![Entities, EventQueue<WorldEvent>], _>(&mut d);
        let dr = d.end();
        assert!(r.is_ok(), "{:?}", r.unwrap_err());
        assert!(dr.is_ok(), "{:?}", dr.unwrap_err());
    }

    #[test]
    #[should_panic(expected = "Error(\"Unknown resource SceneGraph<Model>\", line: 1, column: 250)")]
    fn deserialize_with_extraneous_types() {
        let mut d = serde_json::Deserializer::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/extraneous-types.json"
        )));
        let r = Resources::deserialize_with::<Reg![Entities, EventQueue<WorldEvent>], _>(&mut d);
        let dr = d.end();
        assert!(r.is_ok(), "{:?}", r.unwrap_err());
        assert!(dr.is_ok(), "{:?}", dr.unwrap_err());
    }
}
