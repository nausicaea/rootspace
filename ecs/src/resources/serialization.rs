use crate::{Resource, ResourceRegistry, Resources};
use log::debug;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::marker::PhantomData;

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

fn serialize_entry<SM, R>(
    resources: &Resources,
    serialize_map: &mut SM,
    _: PhantomData<R>,
) -> Result<(), SM::Error>
where
    SM: SerializeMap,
    R: Resource + Serialize,
{
    if resources.contains::<R>() {
        #[cfg(any(test, debug_assertions))]
        debug!("Serializing the resource {}", &std::any::type_name::<R>());
        serialize_map.serialize_entry(
            &std::any::type_name::<R>(),
            &SerContainer::new(&*resources.borrow::<R>()),
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

fn serialize_recursive<SM, RR>(
    resources: &Resources,
    serialize_map: &mut SM,
    _: PhantomData<RR>,
) -> Result<(), SM::Error>
where
    SM: SerializeMap,
    RR: ResourceRegistry,
{
    if RR::LEN > 0 {
        serialize_entry::<SM, RR::Head>(resources, serialize_map, PhantomData::default())?;

        serialize_recursive::<SM, RR::Tail>(resources, serialize_map, PhantomData::default())
    } else {
        Ok(())
    }
}

pub fn serialize_resources<SM, RR>(
    resources: &Resources,
    serialize_map: &mut SM,
) -> Result<(), SM::Error>
where
    SM: SerializeMap,
    RR: ResourceRegistry,
{
    serialize_recursive::<SM, RR>(resources, serialize_map, PhantomData::default())
}
