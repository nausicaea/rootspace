use std::{any::type_name, marker::PhantomData};

use anyhow::Error;
use log::trace;
use try_default::TryDefault;

use super::Resources;
use crate::registry::ResourceRegistry;

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
