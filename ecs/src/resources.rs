//! Provides the resource manager.

use crate::{components::Component, persistence::Persistence, resource::Resource};
use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    ser::{self, Serialize, SerializeMap, SerializeStruct, Serializer},
    Deserialize,
};
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    fmt,
    marker::PhantomData,
};
use typename::TypeName;

macro_rules! count_tts {
    () => { 0usize };
    ($_head:tt $($tail:tt)*) => { 1usize + count_tts!($($tail)*) };
}

macro_rules! impl_ser_with {
    ($name:ident, $($r:tt),+) => {
        /// Serialize `Resources` to a set of specified resource types.
        pub fn $name<$($r),+, S>(&self, serializer: S) -> Result<(), S::Error>
        where
            $(
            $r: Resource + TypeName + Serialize,
            )+
            S: Serializer,
        {
            struct SerContainer<'a, R> {
                persistence: Persistence,
                resource: &'a R,
            }

            impl<'a, R> SerContainer<'a, R> {
                fn new(p: Persistence, r: &'a R) -> Self {
                    SerContainer { persistence: p, resource: r }
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
                    let mut state = serializer.serialize_struct("SerContainer", 2)?;
                    state.serialize_field("persistence", &self.persistence)?;
                    state.serialize_field("resource", self.resource)?;
                    state.end()
                }
            }

            let mut state = serializer.serialize_map(Some(count_tts!($($r)+)))?;
            $(
            if self.has::<$r>() {
                state.serialize_entry(&$r::type_name(), &SerContainer::new(self.persistence_of::<$r>(), &*self.borrow::<$r>()))?;
            } else {
                return Err(ser::Error::custom(format!("resource {} was not found", $r::type_name())));
            }
            )+
            state.end()?;
            Ok(())
        }
    };
}

macro_rules! impl_de_with {
    ($name:ident, $($r:tt),+) => {
        /// Deserialize `Resources` from a set of specified resource types.
        pub fn $name<'de, $($r),+, D>(deserializer: D) -> Result<Self, D::Error>
        where
            $(
            $r: Resource + TypeName + Deserialize<'de>,
            )+
            D: Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct DeContainer<R> {
                persistence: Persistence,
                resource: R,
            }

            struct ResourcesVisitor<$($r),+> {
                _r: PhantomData<($($r),+)>,
            }

            impl<$($r),+> Default for ResourcesVisitor<$($r),+> {
                fn default() -> Self {
                    ResourcesVisitor {
                        _r: PhantomData::default(),
                    }
                }
            }

            impl<'de, $($r),+> Visitor<'de> for ResourcesVisitor<$($r),+>
            where
                $(
                $r: Resource + TypeName + Deserialize<'de>,
                )+
            {
                type Value = Resources;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "a map of type names and their serialized data including a persistence marker")
                }

                fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    let mut resources = Resources::with_capacity(access.size_hint().unwrap_or(count_tts!($($r)+)));

                    while let Some(key) = access.next_key()? {
                        if false {
                            // This clause is just here to make the macro easier to write.
                        }
                        $(
                        else if key == <$r as TypeName>::type_name() {
                            let c = access.next_value::<DeContainer<$r>>()?;
                            resources.insert(c.resource, c.persistence);
                        }
                        )+
                        else {
                            return Err(de::Error::unknown_field(key, &[]));
                        }
                    }

                    Ok(resources)
                }
            }

            deserializer.deserialize_map(ResourcesVisitor::<$($r),+>::default())
        }
    };
}

macro_rules! impl_de_additive_with {
    ($name:ident, $deser_name:ident, $($r:tt),+) => {
        /// Deserialize a set of resource types into an existing instance of `Resources`.
        pub fn $name<'de, $($r),+, D>(&mut self, deserializer: D) -> Result<(), D::Error>
        where
            $(
            $r: Resource + TypeName + Deserialize<'de>,
            )+
            D: Deserializer<'de>,
        {
            let resources = Resources::$deser_name::<$($r),+, D>(deserializer)?;
            self.join(resources);
            Ok(())
        }
    };
}

/// A container that manages resources. Allows mutable borrows of multiple different resources at
/// the same time.
#[derive(Default, Debug)]
pub struct Resources {
    resources: HashMap<TypeId, RefCell<Box<dyn Resource>>>,
    persistences: HashMap<TypeId, Persistence>,
}

impl Resources {
    impl_ser_with!(serialize_with_1, R0);

    impl_ser_with!(serialize_with_2, R0, R1);

    impl_ser_with!(serialize_with_3, R0, R1, R2);

    impl_ser_with!(serialize_with_4, R0, R1, R2, R3);

    impl_ser_with!(serialize_with_5, R0, R1, R2, R3, R4);

    impl_ser_with!(serialize_with_6, R0, R1, R2, R3, R4, R5);

    impl_ser_with!(serialize_with_7, R0, R1, R2, R3, R4, R5, R6);

    impl_ser_with!(serialize_with_8, R0, R1, R2, R3, R4, R5, R6, R7);

    impl_de_with!(deserialize_with_1, R0);

    impl_de_with!(deserialize_with_2, R0, R1);

    impl_de_with!(deserialize_with_3, R0, R1, R2);

    impl_de_with!(deserialize_with_4, R0, R1, R2, R3);

    impl_de_with!(deserialize_with_5, R0, R1, R2, R3, R4);

    impl_de_with!(deserialize_with_6, R0, R1, R2, R3, R4, R5);

    impl_de_with!(deserialize_with_7, R0, R1, R2, R3, R4, R5, R6);

    impl_de_with!(deserialize_with_8, R0, R1, R2, R3, R4, R5, R6, R7);

    impl_de_additive_with!(deserialize_additive_with_1, deserialize_with_1, R0);

    impl_de_additive_with!(deserialize_additive_with_2, deserialize_with_2, R0, R1);

    impl_de_additive_with!(deserialize_additive_with_3, deserialize_with_3, R0, R1, R2);

    impl_de_additive_with!(deserialize_additive_with_4, deserialize_with_4, R0, R1, R2, R3);

    impl_de_additive_with!(deserialize_additive_with_5, deserialize_with_5, R0, R1, R2, R3, R4);

    impl_de_additive_with!(deserialize_additive_with_6, deserialize_with_6, R0, R1, R2, R3, R4, R5);

    impl_de_additive_with!(
        deserialize_additive_with_7,
        deserialize_with_7,
        R0,
        R1,
        R2,
        R3,
        R4,
        R5,
        R6
    );

    impl_de_additive_with!(
        deserialize_additive_with_8,
        deserialize_with_8,
        R0,
        R1,
        R2,
        R3,
        R4,
        R5,
        R6,
        R7
    );

    /// Create a new, empty resources container.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new resources container with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Resources {
            resources: HashMap::with_capacity(cap),
            persistences: HashMap::with_capacity(cap),
        }
    }

    /// Join the resources from another container.
    pub fn join(&mut self, resources: Self) {
        for (k, v) in resources.resources {
            self.resources.insert(k, v);
        }
        for (k, v) in resources.persistences {
            self.persistences.insert(k, v);
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
        R: Resource + TypeName,
    {
        self.resources.insert(TypeId::of::<R>(), RefCell::new(Box::new(res)));
        self.persistences.insert(TypeId::of::<R>(), persistence);
    }

    /// Removes the resource of the specified type.
    pub fn remove<R>(&mut self)
    where
        R: Resource + TypeName,
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
        *self
            .persistences
            .get(&TypeId::of::<R>())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, TypeName, Serialize, Deserialize)]
    struct TestResourceA(usize);

    impl Resource for TestResourceA {}

    #[derive(Debug, Default, TypeName, Serialize, Deserialize)]
    struct TestResourceB(f32);

    impl Resource for TestResourceB {}

    #[derive(Debug, Default, TypeName, Serialize, Deserialize)]
    struct TestResourceC(String);

    impl Resource for TestResourceC {}

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
        resources.insert(TestResourceA::default(), Persistence::Runtime);
    }

    #[test]
    fn borrow() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA::default(), Persistence::Runtime);

        let _: Ref<TestResourceA> = resources.borrow();
    }

    #[test]
    fn serialize() {
        let mut resources = Resources::default();
        resources.insert(TestResourceA::default(), Persistence::Runtime);
        resources.insert(TestResourceB::default(), Persistence::None);

        let mut writer: Vec<u8> = Vec::with_capacity(128);
        let mut s = serde_json::Serializer::new(&mut writer);
        assert!(resources.serialize_with_1::<TestResourceC, _>(&mut s).is_err());

        let mut writer: Vec<u8> = Vec::with_capacity(128);
        let mut s = serde_json::Serializer::new(&mut writer);
        assert!(resources.serialize_with_1::<TestResourceA, _>(&mut s).is_ok());
        assert_eq!(
            unsafe { String::from_utf8_unchecked(writer) },
            "{\"ecs::resources::tests::TestResourceA\":{\"persistence\":\"Runtime\",\"resource\":0}}"
        );

        let mut writer: Vec<u8> = Vec::with_capacity(128);
        let mut s = serde_json::Serializer::new(&mut writer);
        assert!(resources
            .serialize_with_2::<TestResourceA, TestResourceB, _>(&mut s)
            .is_ok());
        assert_eq!(unsafe { String::from_utf8_unchecked(writer) }, "{\"ecs::resources::tests::TestResourceA\":{\"persistence\":\"Runtime\",\"resource\":0},\"ecs::resources::tests::TestResourceB\":{\"persistence\":\"None\",\"resource\":0.0}}");
    }

    #[test]
    fn deserialize() {
        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":{\"persistence\":\"Runtime\",\"resource\":0}}",
        );
        let resources = Resources::deserialize_with_1::<TestResourceA, _>(&mut d).unwrap();
        assert!(d.end().is_ok());
        let _: Ref<TestResourceA> = resources.borrow();

        let mut d = serde_json::Deserializer::from_slice(b"{\"ecs::resources::tests::TestResourceA\":{\"persistence\":\"Runtime\",\"resource\":0},\"ecs::resources::tests::TestResourceB\":{\"persistence\":\"None\",\"resource\":0.0}}");
        let resources = Resources::deserialize_with_2::<TestResourceA, TestResourceB, _>(&mut d).unwrap();
        assert!(d.end().is_ok());
        let _: Ref<TestResourceA> = resources.borrow();
        let _: Ref<TestResourceB> = resources.borrow();
    }

    #[test]
    fn deserialize_additive() {
        let mut resources = Resources::default();

        let mut d = serde_json::Deserializer::from_slice(
            b"{\"ecs::resources::tests::TestResourceA\":{\"persistence\":\"Runtime\",\"resource\":0}}",
        );
        resources
            .deserialize_additive_with_1::<TestResourceA, _>(&mut d)
            .unwrap();
        assert!(d.end().is_ok());
        let _: Ref<TestResourceA> = resources.borrow();
    }
}
