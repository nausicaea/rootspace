use crate::{Resource, ResourceRegistry, Resources};
use log::debug;
use serde::{ser::{self, SerializeMap}, Serialize, Serializer};
use std::marker::PhantomData;
use crate::short_type_name::short_type_name;

struct SerResource<'a, R>(&'a R);

impl<'a, R> SerResource<'a, R> {
    fn new(r: &'a R) -> Self {
        SerResource(r)
    }
}

impl<'a, R> Serialize for SerResource<'a, R>
where
    R: Resource + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

fn serialize_recursive<RR, SM>(
    resources: &Resources,
    serialize_map: &mut SM,
    _: PhantomData<RR>,
) -> Result<(), SM::Error>
where
    SM: SerializeMap,
    RR: ResourceRegistry,
{
    if RR::LEN == 0 {
        return Ok(())
    }

    if !resources.contains::<RR::Head>() {
        return Err(ser::Error::custom(format!(
            "the resource {} was not found",
            short_type_name::<RR::Head>(),
        )))
    }

    #[cfg(any(test, debug_assertions))]
    debug!("Serializing the resource {}", &short_type_name::<RR::Head>());
    serialize_map.serialize_entry(
        &short_type_name::<RR::Head>(),
        &SerResource::new(&*resources.borrow::<RR::Head>()),
    )?;

    serialize_recursive::<RR::Tail, SM>(resources, serialize_map, PhantomData::default())
}

pub struct SerResources<'a, RR> {
    resources: &'a Resources,
    _rr: PhantomData<RR>,
}

impl<'a, RR> From<&'a Resources> for SerResources<'a, RR>
    where
        RR: ResourceRegistry,
{
    fn from(resources: &'a Resources) -> Self {
        SerResources {
            resources,
            _rr: PhantomData::default(),
        }
    }
}

impl<'a, RR> Serialize for SerResources<'a, RR>
    where
        RR: ResourceRegistry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(RR::LEN))?;

        serialize_recursive::<RR, S::SerializeMap>(
            self.resources,
            &mut state,
            PhantomData::default()
        )?;

        state.end()
    }
}

