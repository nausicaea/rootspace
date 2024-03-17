use crate::glamour::mat::Mat4;
use crate::glamour::quat::Quat;
use crate::glamour::unit::Unit;
use crate::glamour::vec::Vec4;
use nalgebra::{RealField, SimdValue};

impl<R> From<Vec4<R>> for nalgebra::Vector4<R> {
    fn from(value: Vec4<R>) -> Self {
        nalgebra::Vector4::new(value.x, value.y, value.z, value.w)
    }
}

impl<R> From<Vec4<R>> for cgmath::Vector4<R> {
    fn from(value: Vec4<R>) -> Self {
        cgmath::Vector4::new(value.x, value.y, value.z, value.w)
    }
}

impl<R> From<Unit<Vec4<R>>> for nalgebra::UnitVector3<R>
where
    R: Copy + SimdValue + RealField,
{
    fn from(value: Unit<Vec4<R>>) -> Self {
        nalgebra::Unit::new_unchecked(nalgebra::Vector3::new(value.x, value.y, value.z))
    }
}

impl<R> From<Unit<Vec4<R>>> for cgmath::Vector3<R>
where
    R: Copy,
{
    fn from(value: Unit<Vec4<R>>) -> Self {
        cgmath::Vector3::new(value.x, value.y, value.z)
    }
}

impl<R> From<Quat<R>> for nalgebra::Quaternion<R> {
    fn from(value: Quat<R>) -> Self {
        nalgebra::Quaternion::new(value.w, value.i, value.j, value.k)
    }
}

impl<R> From<Quat<R>> for cgmath::Quaternion<R> {
    fn from(value: Quat<R>) -> Self {
        cgmath::Quaternion::new(value.w, value.i, value.j, value.k)
    }
}

impl<R> From<Unit<Quat<R>>> for nalgebra::UnitQuaternion<R>
where
    R: SimdValue + RealField,
{
    fn from(value: Unit<Quat<R>>) -> Self {
        nalgebra::UnitQuaternion::from_quaternion(value.0.into())
    }
}

impl<R> From<Unit<Quat<R>>> for cgmath::Quaternion<R> {
    fn from(value: Unit<Quat<R>>) -> Self {
        value.0.into()
    }
}

impl<R> From<Mat4<R>> for nalgebra::Matrix4<R>
where
    R: Copy + nalgebra::Scalar,
{
    /// Nalgebra uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn from(val: Mat4<R>) -> Self {
        nalgebra::Matrix4::from(val.t().0)
    }
}

impl<R> From<Mat4<R>> for cgmath::Matrix4<R>
where
    R: Copy,
{
    /// Cgmath uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn from(val: Mat4<R>) -> Self {
        cgmath::Matrix4::from(val.t().0)
    }
}
