use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use num_traits::Float;

use super::vec::Vec4;
use crate::unit::Unit;

#[derive(Debug, PartialEq, Clone)]
pub struct Ray<R> {
    pub o: Vec4<R>,
    pub d: Unit<Vec4<R>>,
}

impl<R> Ray<R>
where
    R: Float,
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

impl<R> AbsDiffEq for Ray<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.o.abs_diff_eq(&rhs.o, epsilon) && self.d.abs_diff_eq(&rhs.d, epsilon)
    }
}

impl<R> RelativeEq for Ray<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.o.relative_eq(&rhs.o, epsilon, max_relative) && self.d.relative_eq(&rhs.d, epsilon, max_relative)
    }
}

impl<R> UlpsEq for Ray<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.o.ulps_eq(&rhs.o, epsilon, max_ulps) && self.d.ulps_eq(&rhs.d, epsilon, max_ulps)
    }
}
