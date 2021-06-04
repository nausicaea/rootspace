use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
    marker::PhantomData,
};

use either::{Either, Left, Right};
use log::trace;
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

use crate::{registry::SystemRegistry, system::System};

use super::{recursors, Systems};
use crate::{resources::Resources, with_resources::WithResources};

#[derive(Debug)]
pub struct TypedSystems<'a, SR>(Either<&'a Systems, Systems>, PhantomData<SR>);

impl<'a, SR> WithResources for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn with_resources(res: &Resources) -> Self {
        let mut systems = Systems::with_capacity(SR::LEN);

        recursors::initialize_recursive::<SR>(res, &mut systems, PhantomData::default());

        TypedSystems(Right(systems), PhantomData::default())
    }
}

impl<'a, SR> PartialEq for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0
            .as_ref()
            .either(|&ref_lhs_s| ref_lhs_s, |lhs_s| lhs_s)
            .eq(rhs.0.as_ref().either(|&ref_lhs_s| ref_lhs_s, |lhs_s| lhs_s))
    }
}

impl<'a, SR> From<TypedSystems<'a, SR>> for Systems
where
    SR: SystemRegistry,
{
    fn from(value: TypedSystems<SR>) -> Self {
        value.0.unwrap_right()
    }
}

impl<'a, SR> From<&'a Systems> for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn from(systems: &'a Systems) -> Self {
        TypedSystems(Left(systems), PhantomData::default())
    }
}

impl<'a, SR> Serialize for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let size_hint = self.0.as_ref().either(|ref_s| ref_s.len(), |s| s.len());
        let mut state = serializer.serialize_map(Some(size_hint))?;

        recursors::serialize_recursive::<SR, S::SerializeMap>(
            self.0.as_ref().unwrap_left(),
            &mut state,
            PhantomData::default(),
        )?;

        state.end()
    }
}

impl<'de, 'a, SR> Deserialize<'de> for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TypedSystemsVisitor::default())
    }
}

#[derive(Debug)]
struct TypedSystemsVisitor<'a, SR>(PhantomData<&'a Systems>, PhantomData<SR>);

impl<'a, SR> Default for TypedSystemsVisitor<'a, SR>
where
    SR: SystemRegistry,
{
    fn default() -> Self {
        TypedSystemsVisitor(PhantomData::default(), PhantomData::default())
    }
}

impl<'de, 'a, SR> Visitor<'de> for TypedSystemsVisitor<'a, SR>
where
    SR: SystemRegistry,
{
    type Value = TypedSystems<'a, SR>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a map of type names to serialized systems")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Do not use `map_access.size_hint()` because we deserialize successfully if and only if
        // all types of the registry are found.
        let mut type_tracker = HashSet::<TypeId>::with_capacity(SR::LEN);
        let mut sys_map = BTreeMap::<usize, Box<dyn System>>::new();

        while let Some(ser_type_name) = map_access.next_key::<String>()? {
            // TODO: Provide a proper list of expected fields based on the complete resource registry

            trace!("Starting deserialization attempt for field {}", ser_type_name);
            recursors::deserialize_recursive::<A, SR>(
                &mut type_tracker,
                &mut sys_map,
                &mut map_access,
                &ser_type_name,
                &[],
                PhantomData::default(),
            )?;
        }

        recursors::validate_recursive::<A, SR>(&type_tracker, PhantomData::default(), PhantomData::default())?;

        let sys_vec: Vec<Box<dyn System>> = sys_map.into_iter().map(|(_, v)| v).collect();

        Ok(TypedSystems(Right(Systems(sys_vec)), PhantomData::default()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde::Deserialize;
    use serde_test::{assert_tokens, Token};

    use crate::{resources::Resources, system::System, Reg, SerializationName};

    use super::*;

    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestSystemA;

    impl System for TestSystemA {
        fn run(&mut self, _: &Resources, _: &Duration, _: &Duration) {}
    }

    impl SerializationName for TestSystemA {}

    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestSystemB;

    impl System for TestSystemB {
        fn run(&mut self, _: &Resources, _: &Duration, _: &Duration) {}
    }

    impl SerializationName for TestSystemB {}

    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestSystemC;

    impl System for TestSystemC {
        fn run(&mut self, _: &Resources, _: &Duration, _: &Duration) {}
    }

    impl SerializationName for TestSystemC {}

    type TypeRegistry = Reg![TestSystemA, TestSystemB, TestSystemC];

    #[test]
    fn serialization_and_deserialization() {
        let res = Resources::with_registry::<Reg![]>().unwrap();
        let sys = Systems::with_registry::<TypeRegistry>(&res);
        let tsys = TypedSystems::<TypeRegistry>::from(&sys);
        assert_tokens(
            &tsys,
            &[
                Token::Map { len: Some(3) },
                Token::Str("TestSystemA"),
                Token::Struct {
                    name: "TypedSystem",
                    len: 2,
                },
                Token::Str("order"),
                Token::U64(0),
                Token::Str("system"),
                Token::UnitStruct { name: "TestSystemA" },
                Token::StructEnd,
                Token::Str("TestSystemB"),
                Token::Struct {
                    name: "TypedSystem",
                    len: 2,
                },
                Token::Str("order"),
                Token::U64(1),
                Token::Str("system"),
                Token::UnitStruct { name: "TestSystemB" },
                Token::StructEnd,
                Token::Str("TestSystemC"),
                Token::Struct {
                    name: "TypedSystem",
                    len: 2,
                },
                Token::Str("order"),
                Token::U64(2),
                Token::Str("system"),
                Token::UnitStruct { name: "TestSystemC" },
                Token::StructEnd,
                Token::MapEnd,
            ],
        );
    }
}
