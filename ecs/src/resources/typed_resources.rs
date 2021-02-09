use either::Either;
use either::Either::{Left, Right};
use super::Resources;
use crate::registry::ResourceRegistry;
use std::marker::PhantomData;
use super::recursors;
use serde::{Serializer, Serialize, ser::SerializeMap, Deserialize, Deserializer};
use serde::de::{Visitor, MapAccess};
use log::debug;

#[derive(Debug)]
pub struct TypedResources<'a, RR>(Either<&'a Resources, Resources>, PhantomData<RR>);

impl<'a, RR> Default for TypedResources<'a, RR>
where
    RR: ResourceRegistry,
{
    fn default() -> Self {
        let mut resources = Resources::with_capacity(RR::LEN);

        recursors::initialize_recursive::<RR>(&mut resources, PhantomData::default());

        TypedResources(Right(resources), PhantomData::default())
    }
}

impl<'a, RR> PartialEq for TypedResources<'a, RR>
    where
        RR: ResourceRegistry,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0.as_ref()
            .either(|&ref_lhs_r| ref_lhs_r, |lhs_r| lhs_r)
            .eq(rhs.0.as_ref().either(|&ref_lhs_r| ref_lhs_r, |lhs_r| lhs_r))
    }
}

impl<'a, RR> From<TypedResources<'a, RR>> for Resources
where
    RR: ResourceRegistry,
{
    fn from(value: TypedResources<RR>) -> Self {
        value.0.unwrap_right()
    }
}

impl<'a, RR> From<&'a Resources> for TypedResources<'a, RR>
where
    RR: ResourceRegistry,
{
    fn from(value: &'a Resources) -> Self {
        TypedResources(Left(value), PhantomData::default())
    }
}

impl<'a, RR> Serialize for TypedResources<'a ,RR>
where
    RR: ResourceRegistry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let size_hint = self.0.as_ref().either(|ref_r| ref_r.len(), |r| r.len());
        let mut state = serializer.serialize_map(Some(size_hint))?;

        recursors::serialize_recursive::<RR, S::SerializeMap>(
            self.0.as_ref().unwrap_left(),
            &mut state,
            PhantomData::default(),
        )?;

        state.end()
    }
}

impl<'de, 'a, RR> Deserialize<'de> for TypedResources<'a, RR>
where
    RR: ResourceRegistry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TypedResourcesVisitor::default())
    }
}

#[derive(Debug)]
struct TypedResourcesVisitor<'a, RR>(PhantomData<&'a Resources>, PhantomData<RR>);

impl<'a, RR> Default for TypedResourcesVisitor<'a, RR>
where
    RR: ResourceRegistry,
{
    fn default() -> Self {
        TypedResourcesVisitor(PhantomData::default(), PhantomData::default())
    }
}

impl<'de, 'a, RR> Visitor<'de> for TypedResourcesVisitor<'a, RR>
where
    RR: ResourceRegistry,
{
    type Value = TypedResources<'a, RR>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a map of type names to serialized resources")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
    {
        // Do not use `map_access.size_hint()` because we deserialize successfully if and only if
        // all types of the registry are found.
        let mut resources = Resources::with_capacity(RR::LEN);

        while let Some(ser_type_name) = map_access.next_key::<&'de str>()? {
            // TODO: Provide a proper list of expected fields based on the complete resource registry
            #[cfg(any(test, debug_assertions))]
            debug!("Starting deser attempt for field {}", ser_type_name);
            recursors::deserialize_recursive::<A, RR>(
                &mut resources,
                &mut map_access,
                &ser_type_name,
                &[],
                PhantomData::default(),
            )?;
        }

        recursors::validate_recursive::<A, RR>(
            &resources,
            PhantomData::default(),
            PhantomData::default()
        )?;

        Ok(TypedResources(Right(resources), PhantomData::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::Resource;
    use serde_test::{assert_tokens, Token};
    use crate::Reg;

    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestResourceA;

    impl Resource for TestResourceA {}

    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestResourceB;

    impl Resource for TestResourceB {}

    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestResourceC;

    impl Resource for TestResourceC {}

    type TypeRegistry = Reg![TestResourceA, TestResourceB, TestResourceC];

    #[test]
    fn serialization_and_deserialization() {
        let res = Resources::with_registry::<TypeRegistry>();
        let tres = TypedResources::<TypeRegistry>::from(&res);
        assert_tokens(
            &tres,
            &[
                Token::Map { len: Some(3) },
                Token::BorrowedStr("TestResourceA"),
                Token::UnitStruct { name: "TestResourceA" },
                Token::BorrowedStr("TestResourceB"),
                Token::UnitStruct { name: "TestResourceB" },
                Token::BorrowedStr("TestResourceC"),
                Token::UnitStruct { name: "TestResourceC" },
                Token::MapEnd,
            ],
        );
    }
}
