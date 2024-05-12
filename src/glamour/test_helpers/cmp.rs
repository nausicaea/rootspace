use crate::{
    glamour::{mat::Mat4, quat::Quat, unit::Unit},
    Vec4,
};

impl<R> PartialEq<nalgebra::Vector4<R>> for Vec4<R>
where
    R: PartialEq,
{
    fn eq(&self, rhs: &nalgebra::Vector4<R>) -> bool {
        self.w.eq(&rhs[3]) && self.x.eq(&rhs[0]) && self.y.eq(&rhs[1]) && self.z.eq(&rhs[2])
    }
}

impl<R> PartialEq<cgmath::Vector4<R>> for Vec4<R>
where
    R: PartialEq,
{
    fn eq(&self, rhs: &cgmath::Vector4<R>) -> bool {
        self.w.eq(&rhs.w) && self.x.eq(&rhs.x) && self.y.eq(&rhs.y) && self.z.eq(&rhs.z)
    }
}

impl<R> PartialEq<nalgebra::Quaternion<R>> for Quat<R>
where
    R: PartialEq,
{
    fn eq(&self, rhs: &nalgebra::Quaternion<R>) -> bool {
        self.w.eq(&rhs.coords[3]) && self.i.eq(&rhs.coords[0]) && self.j.eq(&rhs.coords[1]) && self.k.eq(&rhs.coords[2])
    }
}

impl<R> PartialEq<cgmath::Quaternion<R>> for Quat<R>
where
    R: PartialEq,
{
    fn eq(&self, rhs: &cgmath::Quaternion<R>) -> bool {
        self.w.eq(&rhs.s) && self.i.eq(&rhs.v.x) && self.j.eq(&rhs.v.y) && self.k.eq(&rhs.v.z)
    }
}

impl<R> PartialEq<nalgebra::UnitQuaternion<R>> for Unit<Quat<R>>
where
    R: PartialEq,
{
    fn eq(&self, rhs: &nalgebra::UnitQuaternion<R>) -> bool {
        self.w.eq(&rhs.coords[3]) && self.i.eq(&rhs.coords[0]) && self.j.eq(&rhs.coords[1]) && self.k.eq(&rhs.coords[2])
    }
}

impl<R> PartialEq<cgmath::Quaternion<R>> for Unit<Quat<R>>
where
    R: PartialEq,
{
    fn eq(&self, rhs: &cgmath::Quaternion<R>) -> bool {
        self.w.eq(&rhs.s) && self.i.eq(&rhs.v.x) && self.j.eq(&rhs.v.y) && self.k.eq(&rhs.v.z)
    }
}

impl<R> PartialEq<nalgebra::Matrix4<R>> for Mat4<R>
where
    R: PartialEq,
{
    /// Nalgebra uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn eq(&self, other: &nalgebra::Matrix4<R>) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if self.0[r][c].ne(&other.data.0[c][r]) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> PartialEq<cgmath::Matrix4<R>> for Mat4<R>
where
    R: PartialEq,
{
    /// Cgmath uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn eq(&self, other: &cgmath::Matrix4<R>) -> bool {
        let cgmath_mat: &[[R; 4]; 4] = other.as_ref();
        for r in 0..4 {
            for c in 0..4 {
                if self.0[r][c].ne(&cgmath_mat[c][r]) {
                    return false;
                }
            }
        }
        true
    }
}
