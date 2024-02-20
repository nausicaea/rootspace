use crate::glamour::mat::Mat4;
use crate::glamour::ops::norm::Norm;
use crate::glamour::quat::Quat;
use crate::glamour::vec::Vec4;
use num_traits::Float;

use super::Unit;

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
