use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::{affine::Affine, mat::Mat4, quat::Quat, unit::Unit, vec::Vec4};

impl<R> AbsDiffEq<nalgebra::Vector4<R>> for Vec4<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &nalgebra::Vector4<R>, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs[3], epsilon)
            && self.x.abs_diff_eq(&rhs[0], epsilon)
            && self.y.abs_diff_eq(&rhs[1], epsilon)
            && self.z.abs_diff_eq(&rhs[2], epsilon)
    }
}

impl<R> AbsDiffEq<cgmath::Vector4<R>> for Vec4<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &cgmath::Vector4<R>, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs.w, epsilon)
            && self.x.abs_diff_eq(&rhs.x, epsilon)
            && self.y.abs_diff_eq(&rhs.y, epsilon)
            && self.z.abs_diff_eq(&rhs.z, epsilon)
    }
}

impl<R> RelativeEq<nalgebra::Vector4<R>> for Vec4<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &nalgebra::Vector4<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs[3], epsilon, max_relative)
            && self.x.relative_eq(&rhs[0], epsilon, max_relative)
            && self.y.relative_eq(&rhs[1], epsilon, max_relative)
            && self.z.relative_eq(&rhs[2], epsilon, max_relative)
    }
}

impl<R> RelativeEq<cgmath::Vector4<R>> for Vec4<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &cgmath::Vector4<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs.w, epsilon, max_relative)
            && self.x.relative_eq(&rhs.x, epsilon, max_relative)
            && self.y.relative_eq(&rhs.y, epsilon, max_relative)
            && self.z.relative_eq(&rhs.z, epsilon, max_relative)
    }
}

impl<R> UlpsEq<nalgebra::Vector4<R>> for Vec4<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &nalgebra::Vector4<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs[3], epsilon, max_ulps)
            && self.x.ulps_eq(&rhs[0], epsilon, max_ulps)
            && self.y.ulps_eq(&rhs[1], epsilon, max_ulps)
            && self.z.ulps_eq(&rhs[2], epsilon, max_ulps)
    }
}

impl<R> UlpsEq<cgmath::Vector4<R>> for Vec4<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &cgmath::Vector4<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs.w, epsilon, max_ulps)
            && self.x.ulps_eq(&rhs.x, epsilon, max_ulps)
            && self.y.ulps_eq(&rhs.y, epsilon, max_ulps)
            && self.z.ulps_eq(&rhs.z, epsilon, max_ulps)
    }
}

impl<R> AbsDiffEq<nalgebra::Quaternion<R>> for Quat<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &nalgebra::Quaternion<R>, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs.coords[3], epsilon)
            && self.i.abs_diff_eq(&rhs.coords[0], epsilon)
            && self.j.abs_diff_eq(&rhs.coords[1], epsilon)
            && self.k.abs_diff_eq(&rhs.coords[2], epsilon)
    }
}

impl<R> AbsDiffEq<nalgebra::UnitQuaternion<R>> for Unit<Quat<R>>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &nalgebra::UnitQuaternion<R>, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs.coords[3], epsilon)
            && self.i.abs_diff_eq(&rhs.coords[0], epsilon)
            && self.j.abs_diff_eq(&rhs.coords[1], epsilon)
            && self.k.abs_diff_eq(&rhs.coords[2], epsilon)
    }
}

impl<R> AbsDiffEq<cgmath::Quaternion<R>> for Quat<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &cgmath::Quaternion<R>, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs.s, epsilon)
            && self.i.abs_diff_eq(&rhs.v.x, epsilon)
            && self.j.abs_diff_eq(&rhs.v.y, epsilon)
            && self.k.abs_diff_eq(&rhs.v.z, epsilon)
    }
}

impl<R> AbsDiffEq<cgmath::Quaternion<R>> for Unit<Quat<R>>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &cgmath::Quaternion<R>, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs.s, epsilon)
            && self.i.abs_diff_eq(&rhs.v.x, epsilon)
            && self.j.abs_diff_eq(&rhs.v.y, epsilon)
            && self.k.abs_diff_eq(&rhs.v.z, epsilon)
    }
}

impl<R> RelativeEq<nalgebra::Quaternion<R>> for Quat<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &nalgebra::Quaternion<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs.coords[3], epsilon, max_relative)
            && self.i.relative_eq(&rhs.coords[0], epsilon, max_relative)
            && self.j.relative_eq(&rhs.coords[1], epsilon, max_relative)
            && self.k.relative_eq(&rhs.coords[2], epsilon, max_relative)
    }
}

impl<R> RelativeEq<nalgebra::UnitQuaternion<R>> for Unit<Quat<R>>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &nalgebra::UnitQuaternion<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs.coords[3], epsilon, max_relative)
            && self.i.relative_eq(&rhs.coords[0], epsilon, max_relative)
            && self.j.relative_eq(&rhs.coords[1], epsilon, max_relative)
            && self.k.relative_eq(&rhs.coords[2], epsilon, max_relative)
    }
}

impl<R> RelativeEq<cgmath::Quaternion<R>> for Quat<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &cgmath::Quaternion<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs.s, epsilon, max_relative)
            && self.i.relative_eq(&rhs.v.x, epsilon, max_relative)
            && self.j.relative_eq(&rhs.v.y, epsilon, max_relative)
            && self.k.relative_eq(&rhs.v.z, epsilon, max_relative)
    }
}

impl<R> RelativeEq<cgmath::Quaternion<R>> for Unit<Quat<R>>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &cgmath::Quaternion<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs.s, epsilon, max_relative)
            && self.i.relative_eq(&rhs.v.x, epsilon, max_relative)
            && self.j.relative_eq(&rhs.v.y, epsilon, max_relative)
            && self.k.relative_eq(&rhs.v.z, epsilon, max_relative)
    }
}

impl<R> UlpsEq<nalgebra::Quaternion<R>> for Quat<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &nalgebra::Quaternion<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs.coords[3], epsilon, max_ulps)
            && self.i.ulps_eq(&rhs.coords[0], epsilon, max_ulps)
            && self.j.ulps_eq(&rhs.coords[1], epsilon, max_ulps)
            && self.k.ulps_eq(&rhs.coords[2], epsilon, max_ulps)
    }
}

impl<R> UlpsEq<nalgebra::UnitQuaternion<R>> for Unit<Quat<R>>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &nalgebra::UnitQuaternion<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs.coords[3], epsilon, max_ulps)
            && self.i.ulps_eq(&rhs.coords[0], epsilon, max_ulps)
            && self.j.ulps_eq(&rhs.coords[1], epsilon, max_ulps)
            && self.k.ulps_eq(&rhs.coords[2], epsilon, max_ulps)
    }
}

impl<R> UlpsEq<cgmath::Quaternion<R>> for Quat<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &cgmath::Quaternion<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs.s, epsilon, max_ulps)
            && self.i.ulps_eq(&rhs.v.x, epsilon, max_ulps)
            && self.j.ulps_eq(&rhs.v.y, epsilon, max_ulps)
            && self.k.ulps_eq(&rhs.v.z, epsilon, max_ulps)
    }
}

impl<R> UlpsEq<cgmath::Quaternion<R>> for Unit<Quat<R>>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &cgmath::Quaternion<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs.s, epsilon, max_ulps)
            && self.i.ulps_eq(&rhs.v.x, epsilon, max_ulps)
            && self.j.ulps_eq(&rhs.v.y, epsilon, max_ulps)
            && self.k.ulps_eq(&rhs.v.z, epsilon, max_ulps)
    }
}

impl<R> AbsDiffEq<nalgebra::Matrix4<R>> for Mat4<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    /// Nalgebra uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn abs_diff_eq(&self, rhs: &nalgebra::Matrix4<R>, epsilon: R::Epsilon) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if self.0[r][c].abs_diff_ne(&rhs.data.0[c][r], epsilon) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> AbsDiffEq<cgmath::Matrix4<R>> for Mat4<R>
where
    R: AbsDiffEq,
    R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    /// Cgmath uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn abs_diff_eq(&self, rhs: &cgmath::Matrix4<R>, epsilon: R::Epsilon) -> bool {
        let cgmath_mat: &[[R; 4]; 4] = rhs.as_ref();
        #[allow(clippy::needless_range_loop)]
        for r in 0..4 {
            #[allow(clippy::needless_range_loop)]
            for c in 0..4 {
                if self.0[r][c].abs_diff_ne(&cgmath_mat[c][r], epsilon) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> RelativeEq<nalgebra::Matrix4<R>> for Mat4<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    /// Nalgebra uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn relative_eq(&self, rhs: &nalgebra::Matrix4<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if self.0[r][c].relative_ne(&rhs.data.0[c][r], epsilon, max_relative) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> RelativeEq<cgmath::Matrix4<R>> for Mat4<R>
where
    R: RelativeEq,
    R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    /// Cgmath uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn relative_eq(&self, rhs: &cgmath::Matrix4<R>, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        let cgmath_mat: &[[R; 4]; 4] = rhs.as_ref();
        #[allow(clippy::needless_range_loop)]
        for r in 0..4 {
            #[allow(clippy::needless_range_loop)]
            for c in 0..4 {
                if self.0[r][c].relative_ne(&cgmath_mat[c][r], epsilon, max_relative) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> UlpsEq<nalgebra::Matrix4<R>> for Mat4<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    /// Nalgebra uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn ulps_eq(&self, rhs: &nalgebra::Matrix4<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        #[allow(clippy::needless_range_loop)]
        for r in 0..4 {
            #[allow(clippy::needless_range_loop)]
            for c in 0..4 {
                if self.0[r][c].ulps_ne(&rhs.data.0[c][r], epsilon, max_ulps) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> UlpsEq<cgmath::Matrix4<R>> for Mat4<R>
where
    R: UlpsEq,
    R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    /// Cgmath uses column-major storage, while glamour uses row-major storage. We need to invert the indexes for proper comparison.
    fn ulps_eq(&self, rhs: &cgmath::Matrix4<R>, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        let cgmath_mat: &[[R; 4]; 4] = rhs.as_ref();
        #[allow(clippy::needless_range_loop)]
        for r in 0..4 {
            #[allow(clippy::needless_range_loop)]
            for c in 0..4 {
                if self.0[r][c].ulps_ne(&cgmath_mat[c][r], epsilon, max_ulps) {
                    return false;
                }
            }
        }
        true
    }
}

impl AbsDiffEq<nalgebra::Similarity3<f32>> for Affine<f32> {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &nalgebra::Similarity3<f32>, epsilon: Self::Epsilon) -> bool {
        self.t.x.abs_diff_eq(&other.isometry.translation.x, epsilon)
            && self.t.y.abs_diff_eq(&other.isometry.translation.y, epsilon)
            && self.t.z.abs_diff_eq(&other.isometry.translation.z, epsilon)
            && self.o.w.abs_diff_eq(&other.isometry.rotation.w, epsilon)
            && self.o.i.abs_diff_eq(&other.isometry.rotation.i, epsilon)
            && self.o.j.abs_diff_eq(&other.isometry.rotation.j, epsilon)
            && self.o.k.abs_diff_eq(&other.isometry.rotation.k, epsilon)
            && self.s.abs_diff_eq(&other.scaling(), epsilon)
    }
}

impl AbsDiffEq<cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>> for Affine<f32> {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }

    fn abs_diff_eq(
        &self,
        other: &cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>,
        epsilon: Self::Epsilon,
    ) -> bool {
        self.t.x.abs_diff_eq(&other.disp.x, epsilon)
            && self.t.y.abs_diff_eq(&other.disp.y, epsilon)
            && self.t.z.abs_diff_eq(&other.disp.z, epsilon)
            && self.o.w.abs_diff_eq(&other.rot.s, epsilon)
            && self.o.i.abs_diff_eq(&other.rot.v.x, epsilon)
            && self.o.j.abs_diff_eq(&other.rot.v.y, epsilon)
            && self.o.k.abs_diff_eq(&other.rot.v.z, epsilon)
            && self.s.abs_diff_eq(&other.scale, epsilon)
    }
}

impl RelativeEq<nalgebra::Similarity3<f32>> for Affine<f32> {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &nalgebra::Similarity3<f32>,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.t
            .x
            .relative_eq(&other.isometry.translation.x, epsilon, max_relative)
            && self
                .t
                .y
                .relative_eq(&other.isometry.translation.y, epsilon, max_relative)
            && self
                .t
                .z
                .relative_eq(&other.isometry.translation.z, epsilon, max_relative)
            && self.o.w.relative_eq(&other.isometry.rotation.w, epsilon, max_relative)
            && self.o.i.relative_eq(&other.isometry.rotation.i, epsilon, max_relative)
            && self.o.j.relative_eq(&other.isometry.rotation.j, epsilon, max_relative)
            && self.o.k.relative_eq(&other.isometry.rotation.k, epsilon, max_relative)
            && self.s.relative_eq(&other.scaling(), epsilon, max_relative)
    }
}

impl RelativeEq<cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>> for Affine<f32> {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.t.x.relative_eq(&other.disp.x, epsilon, max_relative)
            && self.t.y.relative_eq(&other.disp.y, epsilon, max_relative)
            && self.t.z.relative_eq(&other.disp.z, epsilon, max_relative)
            && self.o.w.relative_eq(&other.rot.s, epsilon, max_relative)
            && self.o.i.relative_eq(&other.rot.v.x, epsilon, max_relative)
            && self.o.j.relative_eq(&other.rot.v.y, epsilon, max_relative)
            && self.o.k.relative_eq(&other.rot.v.z, epsilon, max_relative)
            && self.s.relative_eq(&other.scale, epsilon, max_relative)
    }
}

impl UlpsEq<nalgebra::Similarity3<f32>> for Affine<f32> {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }

    fn ulps_eq(&self, other: &nalgebra::Similarity3<f32>, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.t.x.ulps_eq(&other.isometry.translation.x, epsilon, max_ulps)
            && self.t.y.ulps_eq(&other.isometry.translation.y, epsilon, max_ulps)
            && self.t.z.ulps_eq(&other.isometry.translation.z, epsilon, max_ulps)
            && self.o.w.ulps_eq(&other.isometry.rotation.w, epsilon, max_ulps)
            && self.o.i.ulps_eq(&other.isometry.rotation.i, epsilon, max_ulps)
            && self.o.j.ulps_eq(&other.isometry.rotation.j, epsilon, max_ulps)
            && self.o.k.ulps_eq(&other.isometry.rotation.k, epsilon, max_ulps)
            && self.s.ulps_eq(&other.scaling(), epsilon, max_ulps)
    }
}

impl UlpsEq<cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>> for Affine<f32> {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }

    fn ulps_eq(
        &self,
        other: &cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>,
        epsilon: Self::Epsilon,
        max_ulps: u32,
    ) -> bool {
        self.t.x.ulps_eq(&other.disp.x, epsilon, max_ulps)
            && self.t.y.ulps_eq(&other.disp.y, epsilon, max_ulps)
            && self.t.z.ulps_eq(&other.disp.z, epsilon, max_ulps)
            && self.o.w.ulps_eq(&other.rot.s, epsilon, max_ulps)
            && self.o.i.ulps_eq(&other.rot.v.x, epsilon, max_ulps)
            && self.o.j.ulps_eq(&other.rot.v.y, epsilon, max_ulps)
            && self.o.k.ulps_eq(&other.rot.v.z, epsilon, max_ulps)
            && self.s.ulps_eq(&other.scale, epsilon, max_ulps)
    }
}
