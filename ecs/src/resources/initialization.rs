use crate::{MaybeDefault, Resource, ResourceRegistry, Resources};
use log::debug;
use std::marker::PhantomData;
use crate::short_type_name::short_type_name;

fn maybe_default<R>(resources: &mut Resources, _: PhantomData<R>)
where
    R: Resource + MaybeDefault,
{
    if let Some(default_value) = R::maybe_default() {
        #[cfg(any(test, debug_assertions))]
        debug!("Initializing the resource {}", &short_type_name::<R>());
        resources.insert(default_value)
    } else {
        #[cfg(any(test, debug_assertions))]
        debug!(
            "Not initializing the resource {} because it lacks a default constructor",
            &short_type_name::<R>()
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

pub struct InitResources<RR> {
    resources: Resources,
    _rr: PhantomData<RR>,
}

impl<RR> InitResources<RR>
where
    RR: ResourceRegistry,
{
    pub fn new() -> Self {
        let mut resources = Resources::with_capacity(RR::LEN);
        initialize_recursive::<RR>(&mut resources, PhantomData::default());

        InitResources {
            resources,
            _rr: PhantomData::default(),
        }
    }
}

impl<RR> From<InitResources<RR>> for Resources
where
    RR: ResourceRegistry,
{
    fn from(value: InitResources<RR>) -> Self {
        value.resources
    }
}