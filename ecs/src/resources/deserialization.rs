use crate::{ResourceRegistry, Resources};
#[cfg(any(test, debug_assertions))]
use log::debug;
use serde::{
    de,
    de::{MapAccess, Visitor},
};
use std::{fmt, marker::PhantomData};
use serde::{
    de::{Deserializer},
    Deserialize
};
use crate::short_type_name::short_type_name;

fn deserialize_recursive<'de, A, RR>(
    resources: &mut Resources,
    map_access: &mut A,
    type_name: &str,
    expected_type_names: &'static [&'static str],
    _: PhantomData<RR>,
) -> Result<(), A::Error>
where
    A: MapAccess<'de>,
    RR: ResourceRegistry,
{
    if RR::LEN == 0 {
        eprintln!("Reached the end of the registry");
        return Ok(());
    }

    if type_name == short_type_name::<RR::Head>() {
        if resources.contains::<RR::Head>() {
            return Err(de::Error::custom(format!("Duplicate field {}", short_type_name::<RR::Head>())));
        }

        #[cfg(any(test, debug_assertions))]
        debug!("Deserializing the resource {}", short_type_name::<RR::Head>());
        let c = map_access.next_value::<RR::Head>()?;
        resources.insert(c);
        return Ok(());
    }

    #[cfg(any(test, debug_assertions))]
    debug!("Recursing to {}", short_type_name::<RR::Tail>());
    deserialize_recursive::<A, RR::Tail>(
        resources,
        map_access,
        type_name,
        expected_type_names,
        PhantomData::default(),
    )
}

fn validate_recursive<'de, A, RR>(
    resources: &Resources,
    _: PhantomData<A>,
    _: PhantomData<RR>,
) -> Result<(), A::Error>
where
    A: MapAccess<'de>,
    RR: ResourceRegistry,
{
    if RR::LEN == 0 {
        return Ok(());
    }

    if !resources.contains::<RR::Head>() {
        return Err(de::Error::custom(format!("Missing field {}", short_type_name::<RR::Head>())));
    }

    validate_recursive::<A, RR::Tail>(
        resources,
        PhantomData::default(),
        PhantomData::default()
    )
}

pub struct ResourcesVisitor<RR>(PhantomData<RR>);

impl<RR> Default for ResourcesVisitor<RR>
where
    RR: ResourceRegistry,
{
    fn default() -> Self {
        ResourcesVisitor(PhantomData::default())
    }
}

impl<'de, RR> Visitor<'de> for ResourcesVisitor<RR>
where
    RR: ResourceRegistry,
{
    type Value = DeResources<RR>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a map of type names to their serialized data")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Do not use `map_access.size_hint()` because we deserialize successfully if and only if
        // all types of the registry are found.
        let mut resources = Resources::with_capacity(RR::LEN);

        while let Some(type_name) = map_access.next_key::<String>()? {
            // TODO: Provide a proper list of expected fields based on the complete resource registry
            #[cfg(any(test, debug_assertions))]
            debug!("Starting deser attempt for field {}", type_name);
            deserialize_recursive::<A, RR>(
                &mut resources,
                &mut map_access,
                &type_name,
                &[],
                PhantomData::default(),
            )?;
        }

        validate_recursive::<A, RR>(
            &resources,
            PhantomData::default(),
            PhantomData::default()
        )?;

        Ok(DeResources {
            resources,
            _rr: PhantomData::default(),
        })
    }
}

#[derive(Debug)]
pub struct DeResources<RR> {
    resources: Resources,
    _rr: PhantomData<RR>,
}

impl<RR> From<DeResources<RR>> for Resources
    where
        RR: ResourceRegistry,
{
    fn from(value: DeResources<RR>) -> Self {
        value.resources
    }
}

impl<'de, RR> Deserialize<'de> for DeResources<RR>
    where
        RR: ResourceRegistry,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ResourcesVisitor::<RR>::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entities, Reg, VecStorage, Resource, WorldEvent, EventQueue, Resources};
    use serde::de::Deserializer;
    use serde::{Serialize, Deserialize};

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
    fn de_simple() {
        let mut d = serde_json::Deserializer::from_slice(
            b"{\"TestResourceA\":25,\"TestResourceB\":[0,1],\"TestResourceC\":\"Bye\"}",
        );
        let visitor = ResourcesVisitor::<TestRegistry>::default();
        let de_resources = d.deserialize_map(visitor).unwrap();
        let resources = Resources::from(de_resources);
        let dr = d.end();

        assert!(dr.is_ok(), format!("{:?}", dr.unwrap_err()));

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
    fn de_complex() {
        let mut d = serde_json::Deserializer::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/ok.json"
        )));
        let visitor = ResourcesVisitor::<Reg![Entities, EventQueue<WorldEvent>]>::default();
        let r = d.deserialize_map(visitor);
        let dr = d.end();
        assert!(r.is_ok(), format!("{:?}", r.unwrap_err()));
        assert!(dr.is_ok(), format!("{:?}", dr.unwrap_err()));
    }

    #[test]
    #[should_panic]
    fn de_insufficient_types_in_serialization() {
        let mut d = serde_json::Deserializer::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/insufficient-types.json"
        )));
        let visitor = ResourcesVisitor::<Reg![Entities, EventQueue<WorldEvent>]>::default();
        let r = d.deserialize_map(visitor);
        let dr = d.end();
        assert!(r.is_ok(), format!("{:?}", r.unwrap_err()));
        assert!(dr.is_ok(), format!("{:?}", dr.unwrap_err()));
    }

    #[test]
    fn de_extraneous_types_in_serialization() {
        let mut d = serde_json::Deserializer::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/extraneous-types.json"
        )));
        let visitor = ResourcesVisitor::<Reg![Entities, EventQueue<WorldEvent>]>::default();
        let r = d.deserialize_map(visitor);
        let dr = d.end();
        assert!(r.is_ok(), format!("{:?}", r.unwrap_err()));
        assert!(dr.is_ok(), format!("{:?}", dr.unwrap_err()));
    }

    #[test]
    #[should_panic]
    fn de_with_entities() {
        pub type TestRegistry = Reg![Entities,];

        let visitor = ResourcesVisitor::<TestRegistry>::default();
        let mut d = serde_json::Deserializer::from_str("{\"BogusType\":null}");
        let _resources = d.deserialize_map(visitor).unwrap();
    }

    #[test]
    #[should_panic]
    fn de_with_vec_storage() {
        pub type TestRegistry = Reg![VecStorage<usize>,];

        let visitor = ResourcesVisitor::<TestRegistry>::default();
        let mut d = serde_json::Deserializer::from_str("{\"BogusType\":null}");
        let _resources = d.deserialize_map(visitor).unwrap();
    }
}
