//! Provides the resource manager.
#![allow(non_snake_case)]

use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use log::debug;
use serde::{
    de::{self, Deserializer},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

use crate::{component::Component, registry::ResourceRegistry, resource::Resource};

mod deserialization;
mod initialization;
mod serialization;

macro_rules! impl_iter_ref {
    ($name:ident, $iter:ident, #reads: $($type:ident),+ $(,)?) => {
        impl_iter_ref!($name, $iter, #reads: $($type),+, #writes: );
    };

    ($name:ident, $iter:ident, #writes: $($typemut:ident),+ $(,)?) => {
        impl_iter_ref!($name, $iter, #reads: , #writes: $($typemut),+);
    };

    ($name:ident, $iter:ident, #reads: $($type:ident),*, #writes: $($typemut:ident),* $(,)?) => {
        /// Creates a joined iterator over the specified group of components. In other words, only
        /// entities that have all the specified components will be iterated over.
        pub fn $name<$($type,)* $($typemut,)*>(&self) -> $crate::storage::iterators::$iter<$($type::Storage,)* $($typemut::Storage,)*>
        where
            $(
                $type: Component,
            )*
            $(
                $typemut: Component,
            )*
        {
            $(
                let $type = self.borrow::<$type::Storage>();
            )*
            $(
                let $typemut = self.borrow_mut::<$typemut::Storage>();
            )*

            $crate::storage::iterators::$iter::new($($type,)* $($typemut,)*)
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepOriginal,
    Overwrite,
    Fail,
}

/// A container that manages resources. Allows mutable borrows of multiple different resources at
/// the same time.
#[derive(Default, Debug)]
pub struct Resources {
    resources: HashMap<TypeId, RefCell<Box<dyn Resource>>>,
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
        Resources {
            resources: HashMap::with_capacity(cap),
        }
    }

    /// Clears the resources container.
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Insert a new resource.
    pub fn insert<R>(&mut self, res: R)
    where
        R: Resource,
    {
        self.resources
            .insert(TypeId::of::<R>(), RefCell::new(Box::new(res)));
    }

    /// Removes the resource of the specified type.
    pub fn remove<R>(&mut self)
    where
        R: Resource,
    {
        self.resources.remove(&TypeId::of::<R>());
    }

    /// Returns `true` if a resource of the specified type is present.
    pub fn contains<R>(&self) -> bool
    where
        R: Resource,
    {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    /// Borrows the requested resource.
    pub fn borrow<R>(&self) -> Ref<R>
    where
        R: Resource,
    {
        self.resources
            .get(&TypeId::of::<R>())
            .map(|r| {
                Ref::map(r.borrow(), |i| {
                    i.downcast_ref::<R>().expect(&format!(
                        "Could not downcast the requested resource to type {}",
                        std::any::type_name::<R>()
                    ))
                })
            })
            .expect(&format!(
                "Could not find any resource of type {}",
                std::any::type_name::<R>()
            ))
    }

    /// Mutably borrows the requested resource (with a runtime borrow check).
    pub fn borrow_mut<R>(&self) -> RefMut<R>
    where
        R: Resource,
    {
        self.resources
            .get(&TypeId::of::<R>())
            .map(|r| {
                RefMut::map(r.borrow_mut(), |i| {
                    i.downcast_mut::<R>().expect(&format!(
                        "Could not downcast the requested resource to type {}",
                        std::any::type_name::<R>()
                    ))
                })
            })
            .expect(&format!(
                "Could not find any resource of type {}",
                std::any::type_name::<R>()
            ))
    }

    /// Mutably borrows the requested resource (with a compile-time borrow check).
    pub fn get_mut<R>(&mut self) -> &mut R
    where
        R: Resource,
    {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .map(|r| {
                r.get_mut().downcast_mut::<R>().expect(&format!(
                    "Could not downcast the requested resource to type {}",
                    std::any::type_name::<R>()
                ))
            })
            .expect(&format!(
                "Could not find any resource of type {}",
                std::any::type_name::<R>()
            ))
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

    /// In a similar fashion to Resources::serialize, the following section uses the types stored
    /// in the registry to initialize those resources that have a default, parameterless
    /// constructor.
    pub fn initialize<RR>(&mut self)
    where
        RR: ResourceRegistry,
    {
        #[cfg(any(test, debug_assertions))]
        debug!("Beginning the initialization of Resources");
        initialization::initialize_resources::<RR>(self);
        #[cfg(any(test, debug_assertions))]
        debug!("Completed the initialization of Resources");
    }

    /// Serialize the types supplied in the registry from `Resources`.
    pub fn serialize<RR, S>(&self, serializer: S) -> Result<(), S::Error>
    where
        RR: ResourceRegistry,
        S: Serializer,
    {
        #[cfg(any(test, debug_assertions))]
        debug!("Beginning the serialization of Resources");
        let mut state = serializer.serialize_map(Some(RR::LEN))?;
        serialization::serialize_resources::<S::SerializeMap, RR>(self, &mut state)?;
        state.end()?;
        #[cfg(any(test, debug_assertions))]
        debug!("Completed the serialization of Resources");
        Ok(())
    }

    /// Deserialize `Resources` with the provided type registry.
    pub fn deserialize<'de, RR, D>(deserializer: D) -> Result<Self, D::Error>
    where
        RR: ResourceRegistry,
        D: Deserializer<'de>,
    {
        #[cfg(any(test, debug_assertions))]
        debug!("Beginning the deserialization of Resources");
        let resources =
            deserializer.deserialize_map(deserialization::ResourcesVisitor::<RR>::default())?;
        #[cfg(any(test, debug_assertions))]
        debug!("Completed the deserialization of Resources");
        Ok(resources)
    }

    /// Deserialize `Resources` with the provided type registry, by adding the deserialized
    /// resources to existing ones in `Resources`.
    pub fn deserialize_additive<'de, RR, D>(
        &mut self,
        deserializer: D,
        method: ConflictResolution,
    ) -> Result<(), D::Error>
    where
        RR: ResourceRegistry,
        D: Deserializer<'de>,
    {
        #[cfg(any(test, debug_assertions))]
        debug!("Beginning the additive deserialization of Resources");
        let other = Resources::deserialize::<RR, D>(deserializer)?;
        for (k, v) in other.resources {
            if self.resources.contains_key(&k) {
                #[cfg(any(test, debug_assertions))]
                debug!("The new resource {:?} conflicts with a previous one", v);
                match method {
                    ConflictResolution::KeepOriginal => (),
                    ConflictResolution::Overwrite => {
                        self.resources.insert(k, v);
                    }
                    ConflictResolution::Fail => {
                        return Err(de::Error::custom(format!(
                            "Cannot deserialize the resource {:?} because of conflicts",
                            v
                        )))
                    }
                }
            } else {
                #[cfg(any(test, debug_assertions))]
                debug!("Deserializing the resource {:?}", v);
                self.resources.insert(k, v);
            }
        }
        debug!("Completed the additive deserialization of Resources");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json;

    use crate::{world::event::WorldEvent, Entities, EventQueue, Reg};

    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceA(usize);

    impl Resource for TestResourceA {}

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceB(Vec<usize>);

    impl Resource for TestResourceB {}

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceC(String);

    impl Resource for TestResourceC {}

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
    fn serialize() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA(25));
        resources.insert(TestResourceB(Vec::new()));
        resources.insert(TestResourceC(String::from("Bye")));

        let mut writer: Vec<u8> = Vec::with_capacity(128);
        let mut s = serde_json::Serializer::new(&mut writer);
        assert!(resources.serialize::<TestRegistry, _>(&mut s).is_ok());
        assert_eq!(
            unsafe { String::from_utf8_unchecked(writer) },
            "{\"ecs::resources::tests::TestResourceA\":25,\"ecs::resources::tests::TestResourceB\":[],\"ecs::resources::tests::TestResourceC\":\"Bye\"}"
        );
    }

    #[test]
    fn deserialize() {
        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":25,\"ecs::resources::tests::TestResourceB\":[0,1],\"ecs::resources::tests::TestResourceC\":\"Bye\"}",
        );
        let resources = Resources::deserialize::<TestRegistry, _>(&mut d).unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(25));
        assert_eq!(
            *resources.borrow::<TestResourceB>(),
            TestResourceB(vec![0, 1])
        );
        assert_eq!(
            *resources.borrow::<TestResourceC>(),
            TestResourceC(String::from("Bye"))
        );
    }

    #[test]
    fn deserialize_additive_no_overwrite() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA(25));
        resources.insert(TestResourceB(Vec::new()));
        resources.insert(TestResourceC(String::from("Bye")));

        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":100,\"ecs::resources::tests::TestResourceB\":[0,1,2],\"ecs::resources::tests::TestResourceC\":\"Hello, World!\"}",
        );
        resources
            .deserialize_additive::<TestRegistry, _>(&mut d, ConflictResolution::KeepOriginal)
            .unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(25));
        assert_eq!(
            *resources.borrow::<TestResourceB>(),
            TestResourceB(Vec::new())
        );
        assert_eq!(
            *resources.borrow::<TestResourceC>(),
            TestResourceC(String::from("Bye"))
        );
    }

    #[test]
    fn deserialize_additive_overwrite() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA(25));
        resources.insert(TestResourceB(Vec::new()));
        resources.insert(TestResourceC(String::from("Bye")));

        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":100,\"ecs::resources::tests::TestResourceB\":[0,1,2],\"ecs::resources::tests::TestResourceC\":\"Hello, World!\"}",
        );
        resources
            .deserialize_additive::<TestRegistry, _>(&mut d, ConflictResolution::Overwrite)
            .unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(100));
        assert_eq!(
            *resources.borrow::<TestResourceB>(),
            TestResourceB(vec![0, 1, 2])
        );
        assert_eq!(
            *resources.borrow::<TestResourceC>(),
            TestResourceC(String::from("Hello, World!"))
        );
    }

    #[test]
    fn deserialize_additive_partial() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA(25));

        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceB\":[0,1,2],\"ecs::resources::tests::TestResourceC\":\"Hello, World!\"}",
        );
        resources
            .deserialize_additive::<TestRegistry, _>(&mut d, ConflictResolution::KeepOriginal)
            .unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(25));
        assert_eq!(
            *resources.borrow::<TestResourceB>(),
            TestResourceB(vec![0, 1, 2])
        );
        assert_eq!(
            *resources.borrow::<TestResourceC>(),
            TestResourceC(String::from("Hello, World!"))
        );
    }

    #[test]
    fn deserialize_complex() {
        let mut d = serde_json::Deserializer::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/ok.json"
        )));
        let r = Resources::deserialize::<Reg![Entities, EventQueue<WorldEvent>], _>(&mut d);
        let dr = d.end();
        assert!(r.is_ok(), format!("{:?}", r.unwrap_err()));
        assert!(dr.is_ok(), format!("{:?}", dr.unwrap_err()));
    }

    #[test]
    #[should_panic]
    fn deserialize_complex_extraneous_types() {
        let mut d = serde_json::Deserializer::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/extraneous-types.json"
        )));
        let r = Resources::deserialize::<Reg![Entities, EventQueue<WorldEvent>], _>(&mut d);
        let dr = d.end();
        assert!(r.is_ok(), format!("{:?}", r.unwrap_err()));
        assert!(dr.is_ok(), format!("{:?}", dr.unwrap_err()));
    }
}
