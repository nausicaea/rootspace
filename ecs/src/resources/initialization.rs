use crate::{MaybeDefault, Resource, ResourceRegistry, Resources};
use log::debug;
use std::marker::PhantomData;

fn maybe_default<R>(resources: &mut Resources, _: PhantomData<R>)
where
    R: Resource + MaybeDefault,
{
    if let Some(default_value) = R::maybe_default() {
        #[cfg(any(test, debug_assertions))]
        debug!("Initializing the resource {}", &std::any::type_name::<R>());
        resources.insert(default_value)
    } else {
        #[cfg(any(test, debug_assertions))]
        debug!(
            "Not initializing the resource {} because it lacks a default constructor",
            &std::any::type_name::<R>()
        );
    }
}

fn initialize_recursive<RR>(resources: &mut Resources, _: PhantomData<RR>)
where
    RR: ResourceRegistry,
{
    if RR::LEN > 0 {
        maybe_default::<RR::Head>(resources, PhantomData::default());

        initialize_recursive::<RR::Tail>(resources, PhantomData::default());
    }
}

pub fn initialize_resources<RR>(resources: &mut Resources)
where
    RR: ResourceRegistry,
{
    // The following lines look super scary but since initialize_recursive() only accesses the
    // type of reg, this should be alright.
    initialize_recursive::<RR>(resources, PhantomData::default())
}
