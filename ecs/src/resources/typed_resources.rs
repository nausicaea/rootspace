use std::marker::PhantomData;

use anyhow::Error;
use either::{
    Either,
    Either::{Left, Right},
};
use try_default::TryDefault;

use super::{recursors, Resources};
use crate::registry::ResourceRegistry;

#[derive(Debug)]
pub struct TypedResources<'a, RR>(Either<&'a Resources, Resources>, PhantomData<RR>);

impl<'a, RR> TryDefault for TypedResources<'a, RR>
where
    RR: ResourceRegistry,
{
    fn try_default() -> Result<Self, Error> {
        let mut resources = Resources::with_capacity(RR::LEN);

        recursors::initialize_recursive::<RR>(&mut resources, PhantomData::default())?;

        Ok(TypedResources(Right(resources), PhantomData::default()))
    }
}

impl<'a, RR> PartialEq for TypedResources<'a, RR>
where
    RR: ResourceRegistry,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0
            .as_ref()
            .either(|&ref_lhs_r| ref_lhs_r, |lhs_r| lhs_r)
            .eq(rhs.0.as_ref().either(|&ref_lhs_r| ref_lhs_r, |lhs_r| lhs_r))
    }
}

impl<'a, RR> From<TypedResources<'a, RR>> for Resources
where
    RR: ResourceRegistry,
{
    fn from(value: TypedResources<RR>) -> Self {
        value.0.unwrap_right()
    }
}

impl<'a, RR> From<&'a Resources> for TypedResources<'a, RR>
where
    RR: ResourceRegistry,
{
    fn from(value: &'a Resources) -> Self {
        TypedResources(Left(value), PhantomData::default())
    }
}
