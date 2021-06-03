use super::Resources;
use crate::{registry::ResourceRegistry, serialization_name::SerializationName};
use anyhow::Error;
use log::trace;
use serde::{de, de::MapAccess, ser, ser::SerializeMap};
use std::{any::type_name, marker::PhantomData};
use try_default::TryDefault;

pub fn initialize_recursive<RR>(resources: &mut Resources, _: PhantomData<RR>) -> Result<(), Error>
where
    RR: ResourceRegistry,
{
    if RR::LEN == 0 {
        return Ok(());
    }

    match RR::Head::try_default() {
        Ok(default_value) => {
            trace!("Initializing the resource {}", type_name::<RR::Head>());
            resources.insert(default_value);
        }
        Err(e) => {
            return Err(e);
        }
    }

    initialize_recursive::<RR::Tail>(resources, PhantomData::default())
}

pub fn serialize_recursive<RR, SM>(
    resources: &Resources,
    serialize_map: &mut SM,
    _: PhantomData<RR>,
) -> Result<(), SM::Error>
where
    SM: SerializeMap,
    RR: ResourceRegistry,
{
    if RR::LEN == 0 {
        return Ok(());
    }

    if !resources.contains::<RR::Head>() {
        return Err(ser::Error::custom(format!(
            "the resource {} was not found",
            type_name::<RR::Head>(),
        )));
    }

    trace!("Serializing the resource {}", type_name::<RR::Head>());
    let resource = resources.borrow::<RR::Head>();
    serialize_map.serialize_entry(&<RR::Head as SerializationName>::name(), &*resource)?;

    serialize_recursive::<RR::Tail, SM>(resources, serialize_map, PhantomData::default())
}

pub fn deserialize_recursive<'de, A, RR>(
    resources: &mut Resources,
    map_access: &mut A,
    ser_type_name: &str,
    expected_type_names: &'static [&'static str],
    _: PhantomData<RR>,
) -> Result<(), A::Error>
where
    A: MapAccess<'de>,
    RR: ResourceRegistry,
{
    if RR::LEN == 0 {
        return Err(de::Error::custom(format!("Unknown resource {}", ser_type_name)));
    }

    if ser_type_name == <RR::Head as SerializationName>::name() {
        if resources.contains::<RR::Head>() {
            return Err(de::Error::custom(format!(
                "Duplicate resource {}",
                type_name::<RR::Head>()
            )));
        }

        trace!("Deserializing the resource {}", type_name::<RR::Head>());
        let resource = map_access.next_value::<RR::Head>()?;
        resources.insert(resource);
        return Ok(());
    }

    deserialize_recursive::<A, RR::Tail>(
        resources,
        map_access,
        ser_type_name,
        expected_type_names,
        PhantomData::default(),
    )
}

pub fn validate_recursive<'de, A, RR>(
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
        return Err(de::Error::custom(format!(
            "Missing resource {}",
            type_name::<RR::Head>()
        )));
    }

    validate_recursive::<A, RR::Tail>(resources, PhantomData::default(), PhantomData::default())
}
