use std::{any::type_name, marker::PhantomData};

use log::trace;

use super::Systems;
use crate::{registry::SystemRegistry, resources::Resources, with_resources::WithResources};

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
