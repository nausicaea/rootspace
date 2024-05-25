use num_traits::Float;

use super::Unit;
use crate::{mat::Mat4, ops::norm::Norm, quat::Quat, vec::Vec4};

impl<R> From<Quat<R>> for Unit<Quat<R>>
where
    Quat<R>: Norm<Output = R>,
    R: Float,
{
    fn from(v: Quat<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}

impl<R> From<Mat4<R>> for Unit<Mat4<R>>
where
    Mat4<R>: Norm<Output = R>,
    R: Float,
{
    fn from(v: Mat4<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}

impl<R> From<Vec4<R>> for Unit<Vec4<R>>
where
    Vec4<R>: Norm<Output = R>,
    R: Float,
{
    fn from(v: Vec4<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}

impl<R> From<Unit<Self>> for Quat<R> {
    fn from(value: Unit<Self>) -> Self {
        value.0
    }
}

impl<R> From<Unit<Self>> for Mat4<R> {
    fn from(value: Unit<Self>) -> Self {
        value.0
    }
}

impl<R> From<Unit<Self>> for Vec4<R> {
    fn from(value: Unit<Self>) -> Self {
        value.0
    }
}


