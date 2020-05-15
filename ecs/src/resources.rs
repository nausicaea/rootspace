//! Provides the resource manager.

use crate::{components::Component, registry::ResourceRegistry, resource::Resource};
use log::debug;
use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt,
    marker::PhantomData,
};

/// A container that manages resources. Allows mutable borrows of multiple different resources at
/// the same time.
#[derive(Default, Debug)]
pub struct Resources {
    resources: HashMap<TypeId, RefCell<Box<dyn Resource>>>,
}

impl Resources {
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
        self.resources.insert(TypeId::of::<R>(), RefCell::new(Box::new(res)));
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
            .expect(&format!("Could not find any resource of type {}", std::any::type_name::<R>()))
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
            .expect(&format!("Could not find any resource of type {}", std::any::type_name::<R>()))
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
            .expect(&format!("Could not find any resource of type {}", std::any::type_name::<R>()))
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

    /// Serialize the types supplied in the registry from `Resources`.
    pub fn serialize<RR, S>(&self, serializer: S) -> Result<(), S::Error>
    where
        RR: ResourceRegistry,
        S: Serializer,
    {
        struct SerContainer<'a, R>(&'a R);

        impl<'a, R> SerContainer<'a, R> {
            fn new(r: &'a R) -> Self {
                SerContainer(r)
            }
        }

        impl<'a, R> Serialize for SerContainer<'a, R>
        where
            R: Resource + Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_newtype_struct("SerContainer", self.0)
            }
        }

        fn serialize_entry<SM, R>(res: &Resources, state: &mut SM, _: &R) -> Result<(), SM::Error>
        where
            SM: SerializeMap,
            R: Resource + Serialize,
        {
            if res.contains::<R>() {
                #[cfg(any(test, debug_assertions))]
                debug!("Serializing the resource {}", &std::any::type_name::<R>());
                state.serialize_entry(
                    &std::any::type_name::<R>(),
                    &SerContainer::new(&*res.borrow::<R>()),
                )?;
            } else {
                #[cfg(any(test, debug_assertions))]
                debug!(
                    "Not serializing the resource {} because it was not present in Resources",
                    &std::any::type_name::<R>()
                );
            }
            Ok(())
        }

        fn recurse<SM, RR>(res: &Resources, state: &mut SM, reg: &RR) -> Result<(), SM::Error>
        where
            SM: SerializeMap,
            RR: ResourceRegistry,
        {
            if RR::LEN > 0 {
                serialize_entry(res, state, reg.head())?;
                recurse(res, state, reg.tail())
            } else {
                Ok(())
            }
        }

        debug!("Beginning the serialization of Resources");
        let mut state = serializer.serialize_map(Some(RR::LEN))?;

        // The following lines look super scary but since recurse() only accesses the type of reg,
        // this should be alright.
        let reg = unsafe { std::mem::MaybeUninit::<RR>::zeroed().assume_init() };
        recurse(self, &mut state, &reg)?;
        std::mem::forget(reg);

        state.end()?;
        debug!("Completed the serialization of Resources");
        Ok(())
    }

    /// Deserialize `Resources` with the provided type registry.
    pub fn deserialize<'de, RR, D>(deserializer: D) -> Result<Self, D::Error>
    where
        RR: ResourceRegistry,
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeContainer<R>(R);

        fn recurse<'de, A, RR>(res: &mut Resources, access: &mut A, key: &str, reg: &RR) -> Result<(), A::Error>
        where
            A: MapAccess<'de>,
            RR: ResourceRegistry,
        {
            fn sub<'de, A, RR, R>(
                res: &mut Resources,
                access: &mut A,
                key: &str,
                reg: &RR,
                _: &R,
            ) -> Result<(), A::Error>
            where
                A: MapAccess<'de>,
                RR: ResourceRegistry,
                R: Resource + Deserialize<'de>,
            {
                if key == std::any::type_name::<R>() {
                    let c = access.next_value::<DeContainer<R>>()?;
                    res.insert(c.0);
                    Ok(())
                } else {
                    recurse(res, access, key, reg.tail())
                }
            }

            if RR::LEN > 0 {
                sub(res, access, key, reg, reg.head())
            } else {
                Err(de::Error::unknown_field(key, &[]))
            }
        }

        struct ResourcesVisitor<RR>(PhantomData<RR>);

        impl<RR> Default for ResourcesVisitor<RR> {
            fn default() -> Self {
                ResourcesVisitor(PhantomData::default())
            }
        }

        impl<'de, RR> Visitor<'de> for ResourcesVisitor<RR>
        where
            RR: ResourceRegistry,
        {
            type Value = Resources;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "a map of type names to their serialized data"
                )
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut resources = Resources::with_capacity(access.size_hint().unwrap_or(RR::LEN));

                // The following lines look super scary but since recurse() only accesses the type of reg,
                // this should be alright.
                let reg = unsafe { std::mem::MaybeUninit::<RR>::zeroed().assume_init() };
                while let Some(key) = access.next_key::<String>()? {
                    #[cfg(any(test, debug_assertions))]
                    debug!("Deserializing the resource {}", &key);
                    recurse(&mut resources, &mut access, &key, &reg)?;
                }
                std::mem::forget(reg);

                Ok(resources)
            }
        }

        debug!("Beginning the deserialization of Resources");
        let resources = deserializer.deserialize_map(ResourcesVisitor::<RR>::default())?;
        debug!("Completed the deserialization of Resources");
        Ok(resources)
    }

    /// Deserialize `Resources` with the provided type registry, by adding the deserialized
    /// resources to existing ones in `Resources`.
    pub fn deserialize_additive<'de, RR, D>(&mut self, deserializer: D, overwrite: bool) -> Result<(), D::Error>
    where
        RR: ResourceRegistry,
        D: Deserializer<'de>,
    {
        debug!("Beginning the additive deserialization of Resources");
        let other = Resources::deserialize::<RR, D>(deserializer)?;
        for (k, v) in other.resources {
            if !self.resources.contains_key(&k) || overwrite {
                #[cfg(not(any(test, debug_assertions)))]
                self.resources.insert(k, v);
                #[cfg(any(test, debug_assertions))]
                {
                    if let Some(old_v) = self.resources.insert(k, v) {
                        debug!("Overwriting the resource {:?}", old_v);
                    }
                }
            } else {
                #[cfg(any(test, debug_assertions))]
                debug!(
                    "Not adding the resource {:?}, because the same type is already present",
                    v
                );
            }
        }
        debug!("Completed the additive deserialization of Resources");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Reg;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceA(usize);

    impl Resource for TestResourceA {}

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    struct TestResourceB(f32);

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
        resources.insert(TestResourceB(0.141));
        resources.insert(TestResourceC(String::from("Bye")));

        let mut writer: Vec<u8> = Vec::with_capacity(128);
        let mut s = serde_json::Serializer::new(&mut writer);
        assert!(resources.serialize::<TestRegistry, _>(&mut s).is_ok());
        assert_eq!(
            unsafe { String::from_utf8_unchecked(writer) },
            "{\"ecs::resources::tests::TestResourceA\":25,\"ecs::resources::tests::TestResourceB\":0.141,\"ecs::resources::tests::TestResourceC\":\"Bye\"}"
        );
    }

    #[test]
    fn deserialize() {
        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":25,\"ecs::resources::tests::TestResourceB\":0.141,\"ecs::resources::tests::TestResourceC\":\"Bye\"}",
        );
        let resources = Resources::deserialize::<TestRegistry, _>(&mut d).unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(25));
        assert_eq!(*resources.borrow::<TestResourceB>(), TestResourceB(0.141));
        assert_eq!(*resources.borrow::<TestResourceC>(), TestResourceC(String::from("Bye")));
    }

    #[test]
    fn deserialize_additive_no_overwrite() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA(25));
        resources.insert(TestResourceB(0.141));
        resources.insert(TestResourceC(String::from("Bye")));

        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":100,\"ecs::resources::tests::TestResourceB\":10.01,\"ecs::resources::tests::TestResourceC\":\"Hello, World!\"}",
        );
        resources
            .deserialize_additive::<TestRegistry, _>(&mut d, false)
            .unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(25));
        assert_eq!(*resources.borrow::<TestResourceB>(), TestResourceB(0.141));
        assert_eq!(*resources.borrow::<TestResourceC>(), TestResourceC(String::from("Bye")));
    }

    #[test]
    fn deserialize_additive_overwrite() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA(25));
        resources.insert(TestResourceB(0.141));
        resources.insert(TestResourceC(String::from("Bye")));

        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":100,\"ecs::resources::tests::TestResourceB\":10.01,\"ecs::resources::tests::TestResourceC\":\"Hello, World!\"}",
        );
        resources.deserialize_additive::<TestRegistry, _>(&mut d, true).unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(100));
        assert_eq!(*resources.borrow::<TestResourceB>(), TestResourceB(10.01));
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
            b"{\"ecs::resources::tests::TestResourceB\":10.01,\"ecs::resources::tests::TestResourceC\":\"Hello, World!\"}",
        );
        resources
            .deserialize_additive::<TestRegistry, _>(&mut d, false)
            .unwrap();
        assert!(d.end().is_ok());
        assert_eq!(*resources.borrow::<TestResourceA>(), TestResourceA(25));
        assert_eq!(*resources.borrow::<TestResourceB>(), TestResourceB(10.01));
        assert_eq!(
            *resources.borrow::<TestResourceC>(),
            TestResourceC(String::from("Hello, World!"))
        );
    }
}
