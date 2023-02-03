use std::iter::Sum;

use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use num_traits::Float;

use crate::{mat::Mat4, ops::norm::Norm, quat::Quat};

#[cfg_attr(any(test, feature = "serde_support"), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(test, feature = "serde_support"), serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Unit<T>(T);

impl<T> std::fmt::Display for Unit<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<T> AsRef<T> for Unit<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::Deref for Unit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AbsDiffEq for Unit<T>
where
    T: AbsDiffEq,
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    fn default_epsilon() -> T::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: T::Epsilon) -> bool {
        self.0.abs_diff_eq(&rhs.0, epsilon)
    }
}

impl<T> RelativeEq for Unit<T>
where
    T: RelativeEq,
    T::Epsilon: Copy,
{
    fn default_max_relative() -> T::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
        self.0.relative_eq(&rhs.0, epsilon, max_relative)
    }
}

impl<T> UlpsEq for Unit<T>
where
    T: UlpsEq,
    T::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool {
        self.0.ulps_eq(&rhs.0, epsilon, max_ulps)
    }
}

impl<R> From<Quat<R>> for Unit<Quat<R>>
where
    R: Float,
{
    fn from(v: Quat<R>) -> Self {
        From::from(&v)
    }
}

impl<'a, R> From<&'a Quat<R>> for Unit<Quat<R>>
where
    R: Float,
{
    fn from(v: &'a Quat<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}

impl<R> From<Mat4<R>> for Unit<Mat4<R>>
where
    R: Float + Sum,
{
    fn from(v: Mat4<R>) -> Self {
        From::from(&v)
    }
}

impl<'a, R> From<&'a Mat4<R>> for Unit<Mat4<R>>
where
    R: Float + Sum,
{
    fn from(v: &'a Mat4<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}
