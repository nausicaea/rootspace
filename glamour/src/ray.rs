use num_traits::Float;
use std::iter::Sum;
use super::mat::Vec4;
use crate::unit::Unit;

#[derive(Debug, PartialEq, Clone)]
pub struct Ray<R> {
    pub o: Vec4<R>,
    pub d: Unit<Vec4<R>>,
}

impl<R> Ray<R> 
where
    R: Float + Sum,
{
    pub fn new<D>(origin: Vec4<R>, direction: D) -> Self 
    where
        D: Into<Unit<Vec4<R>>>,
    {
        Ray {
            o: origin,
            d: direction.into(),
        }
    }
}

impl<R> Ray<R> 
where
    R: Float,
{
    pub fn at(&self, position: &R) -> Vec4<R> {
        &self.o + self.d.as_ref() * position
    }
}

