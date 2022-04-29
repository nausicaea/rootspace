use std::iter::Sum;
use crate::ops::norm::Norm;
use num_traits::Float;
use crate::quat::Quat;
use crate::mat::Mat;

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde_support", serde(transparent))]
#[derive(Debug, Clone, PartialEq)]
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

impl<R, const I: usize, const J: usize> From<Mat<R, I, J>> for Unit<Mat<R, I, J>>
where
    R: Float + Sum,
{
    fn from(v: Mat<R, I, J>) -> Self {
        From::from(&v)
    }
}

impl<'a, R, const I: usize, const J: usize> From<&'a Mat<R, I, J>> for Unit<Mat<R, I, J>>
where
    R: Float + Sum,
{
    fn from(v: &'a Mat<R, I, J>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}
