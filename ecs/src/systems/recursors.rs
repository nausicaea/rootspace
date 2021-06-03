use std::{
    any::{type_name, TypeId},
    collections::{BTreeMap, HashSet},
    marker::PhantomData,
};

use serde::{de, de::MapAccess, ser, ser::SerializeMap};

use crate::{registry::SystemRegistry, resources::Resources, short_type_name::short_type_name, system::System, with_resources::WithResources, SerializationName};
use log::trace;

use super::{typed_system::TypedSystem, Systems};

pub fn initialize_recursive<SR>(resources: &Resources, systems: &mut Systems, _: PhantomData<SR>)
where
    SR: SystemRegistry,
{
    if SR::LEN == 0 {
        return;
    }

    trace!("Initializing the system {}", type_name::<SR::Head>());
    let default_value = SR::Head::with_resources(resources);
    systems.insert(default_value);

    initialize_recursive::<SR::Tail>(resources, systems, PhantomData::default());
}

pub fn serialize_recursive<SR, SM>(
    systems: &Systems,
    serialize_map: &mut SM,
    _: PhantomData<SR>,
) -> Result<(), SM::Error>
where
    SR: SystemRegistry,
    SM: SerializeMap,
{
    if SR::LEN == 0 {
        return Ok(());
    }

    let stn = <SR::Head as SerializationName>::name();

    trace!("Serializing the system {}", &stn);
    if let Some((order, system)) = systems.find_with_position::<SR::Head>() {
        serialize_map.serialize_entry(&stn, &TypedSystem::new(order, system))?;
    } else {
        return Err(ser::Error::custom(format!("the system {} was not found", stn)));
    }

    serialize_recursive::<SR::Tail, SM>(systems, serialize_map, PhantomData::default())
}

pub fn deserialize_recursive<'de, A, SR>(
    type_tracker: &mut HashSet<TypeId>,
    systems: &mut BTreeMap<usize, Box<dyn System>>,
    map_access: &mut A,
    ser_type_name: &str,
    expected_type_names: &'static [&'static str],
    _: PhantomData<SR>,
) -> Result<(), A::Error>
where
    A: MapAccess<'de>,
    SR: SystemRegistry,
{
    if SR::LEN == 0 {
        return Err(de::Error::custom(format!("Unknown field {}", ser_type_name)));
    }

    let stn = <SR::Head as SerializationName>::name();

    if ser_type_name == stn {
        let tid = TypeId::of::<SR::Head>();
        if type_tracker.contains(&tid) {
            return Err(de::Error::custom(format!("Duplicate field {}", stn)));
        }

        trace!("Deserializing the system {}", type_name::<SR::Head>());
        let c = map_access.next_value::<TypedSystem<SR::Head>>()?;
        systems.insert(c.order, Box::new(c.system.unwrap_right()));
        type_tracker.insert(tid);
        return Ok(());
    }

    deserialize_recursive::<A, SR::Tail>(
        type_tracker,
        systems,
        map_access,
        ser_type_name,
        expected_type_names,
        PhantomData::default(),
    )
}

pub fn validate_recursive<'de, A, SR>(
    type_tracker: &HashSet<TypeId>,
    _: PhantomData<A>,
    _: PhantomData<SR>,
) -> Result<(), A::Error>
where
    A: MapAccess<'de>,
    SR: SystemRegistry,
{
    if SR::LEN == 0 {
        return Ok(());
    }

    if !type_tracker.contains(&TypeId::of::<SR::Head>()) {
        return Err(de::Error::custom(format!(
            "Missing field {}",
            short_type_name::<SR::Head>()
        )));
    }

    validate_recursive::<A, SR::Tail>(type_tracker, PhantomData::default(), PhantomData::default())
}
