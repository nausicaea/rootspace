use std::marker::PhantomData;

use either::{Either, Left, Right};

use super::{recursors, Systems};
use crate::{registry::SystemRegistry, resources::Resources, with_resources::WithResources};

#[derive(Debug)]
pub struct TypedSystems<'a, SR>(Either<&'a Systems, Systems>, PhantomData<SR>);

impl<'a, SR> WithResources for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn with_resources(res: &Resources) -> Self {
        let mut systems = Systems::with_capacity(SR::LEN);

        recursors::initialize_recursive::<SR>(res, &mut systems, PhantomData::default());

        TypedSystems(Right(systems), PhantomData::default())
    }
}

impl<'a, SR> PartialEq for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0
            .as_ref()
            .either(|&ref_lhs_s| ref_lhs_s, |lhs_s| lhs_s)
            .eq(rhs.0.as_ref().either(|&ref_lhs_s| ref_lhs_s, |lhs_s| lhs_s))
    }
}

impl<'a, SR> From<TypedSystems<'a, SR>> for Systems
where
    SR: SystemRegistry,
{
    fn from(value: TypedSystems<SR>) -> Self {
        value.0.unwrap_right()
    }
}

impl<'a, SR> From<&'a Systems> for TypedSystems<'a, SR>
where
    SR: SystemRegistry,
{
    fn from(systems: &'a Systems) -> Self {
        TypedSystems(Left(systems), PhantomData::default())
    }
}
