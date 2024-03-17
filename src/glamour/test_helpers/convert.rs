use crate::glamour::mat::Mat4;
use crate::glamour::quat::Quat;
use crate::glamour::vec::Vec4;

#[cfg(test)]
impl<R> From<Vec4<R>> for nalgebra::Vector4<R> {
    fn from(value: Vec4<R>) -> Self {
        nalgebra::Vector4::new(value.x, value.y, value.z, value.w)
    }
}

#[cfg(test)]
impl<R> From<Vec4<R>> for cgmath::Vector4<R> {
    fn from(value: Vec4<R>) -> Self {
        cgmath::Vector4::new(value.x, value.y, value.z, value.w)
    }
}

#[cfg(test)]
impl<R> From<Quat<R>> for nalgebra::Quaternion<R> {
    fn from(value: Quat<R>) -> Self {
        nalgebra::Quaternion::new(value.w, value.i, value.j, value.k)
    }
}

#[cfg(test)]
impl<R> From<Quat<R>> for cgmath::Quaternion<R> {
    fn from(value: Quat<R>) -> Self {
        cgmath::Quaternion::new(value.w, value.i, value.j, value.k)
    }
}

#[cfg(test)]
impl<R> From<Mat4<R>> for nalgebra::Matrix4<R>
    where
        R: Copy + nalgebra::Scalar,
{
    /// Nalgebra uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn from(val: Mat4<R>) -> Self {
        nalgebra::Matrix4::from(val.t().0)
    }
}

#[cfg(test)]
impl<R> From<Mat4<R>> for cgmath::Matrix4<R>
    where
        R: Copy,
{
    /// Cgmath uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn from(val: Mat4<R>) -> Self {
        cgmath::Matrix4::from(val.t().0)
    }
}

