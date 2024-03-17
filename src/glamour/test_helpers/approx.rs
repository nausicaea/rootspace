use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use crate::glamour::affine::Affine;
use crate::glamour::mat::Mat4;
use crate::glamour::persp::Persp;
use crate::glamour::quat::Quat;
use crate::glamour::unit::Unit;
use crate::Vec4;

impl<R> AbsDiffEq for Vec4<R>
    where
        R: AbsDiffEq,
        R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.x.abs_diff_eq(&rhs.x, epsilon)
            && self.y.abs_diff_eq(&rhs.y, epsilon)
            && self.z.abs_diff_eq(&rhs.z, epsilon)
            && self.w.abs_diff_eq(&rhs.w, epsilon)
    }
}

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

impl<R> RelativeEq for Vec4<R>
    where
        R: RelativeEq,
        R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.x.relative_eq(&rhs.x, epsilon, max_relative)
            && self.y.relative_eq(&rhs.y, epsilon, max_relative)
            && self.z.relative_eq(&rhs.z, epsilon, max_relative)
            && self.w.relative_eq(&rhs.w, epsilon, max_relative)
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

impl<R> UlpsEq for Vec4<R>
    where
        R: UlpsEq,
        R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.x.ulps_eq(&rhs.x, epsilon, max_ulps)
            && self.y.ulps_eq(&rhs.y, epsilon, max_ulps)
            && self.z.ulps_eq(&rhs.z, epsilon, max_ulps)
            && self.w.ulps_eq(&rhs.w, epsilon, max_ulps)
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

impl<R> AbsDiffEq for Quat<R>
    where
        R: AbsDiffEq,
        R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.w.abs_diff_eq(&rhs.w, epsilon)
            && self.i.abs_diff_eq(&rhs.i, epsilon)
            && self.j.abs_diff_eq(&rhs.j, epsilon)
            && self.k.abs_diff_eq(&rhs.k, epsilon)
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

impl<R> RelativeEq for Quat<R>
    where
        R: RelativeEq,
        R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.w.relative_eq(&rhs.w, epsilon, max_relative)
            && self.i.relative_eq(&rhs.i, epsilon, max_relative)
            && self.j.relative_eq(&rhs.j, epsilon, max_relative)
            && self.k.relative_eq(&rhs.k, epsilon, max_relative)
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

impl<R> UlpsEq for Quat<R>
    where
        R: UlpsEq,
        R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.w.ulps_eq(&rhs.w, epsilon, max_ulps)
            && self.i.ulps_eq(&rhs.i, epsilon, max_ulps)
            && self.j.ulps_eq(&rhs.j, epsilon, max_ulps)
            && self.k.ulps_eq(&rhs.k, epsilon, max_ulps)
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

impl<R> AbsDiffEq for Mat4<R>
    where
        R: AbsDiffEq,
        R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.0
            .iter()
            .flatten()
            .zip(rhs.0.iter().flatten())
            .all(|(l, r)| l.abs_diff_eq(r, epsilon))
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
        for r in 0..4 {
            for c in 0..4 {
                if self.0[r][c].abs_diff_ne(&cgmath_mat[c][r], epsilon) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> RelativeEq for Mat4<R>
    where
        R: RelativeEq,
        R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.0
            .iter()
            .flatten()
            .zip(rhs.0.iter().flatten())
            .all(|(l, r)| l.relative_eq(r, epsilon, max_relative))
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
        for r in 0..4 {
            for c in 0..4 {
                if self.0[r][c].relative_ne(&cgmath_mat[c][r], epsilon, max_relative) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> UlpsEq for Mat4<R>
    where
        R: UlpsEq,
        R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.0
            .iter()
            .flatten()
            .zip(rhs.0.iter().flatten())
            .all(|(l, r)| l.ulps_eq(r, epsilon, max_ulps))
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
        for r in 0..4 {
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
        for r in 0..4 {
            for c in 0..4 {
                if self.0[r][c].ulps_ne(&cgmath_mat[c][r], epsilon, max_ulps) {
                    return false;
                }
            }
        }
        true
    }
}

impl<R> AbsDiffEq for Affine<R>
    where
        R: AbsDiffEq,
        R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.t.abs_diff_eq(&rhs.t, epsilon)
            && self.o.abs_diff_eq(&rhs.o, epsilon)
            && self.s.abs_diff_eq(&rhs.s, epsilon)
    }
}

impl<R> RelativeEq for Affine<R>
    where
        R: RelativeEq,
        R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.t.relative_eq(&rhs.t, epsilon, max_relative)
            && self.o.relative_eq(&rhs.o, epsilon, max_relative)
            && self.s.relative_eq(&rhs.s, epsilon, max_relative)
    }
}

impl<R> UlpsEq for Affine<R>
    where
        R: UlpsEq,
        R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.t.ulps_eq(&rhs.t, epsilon, max_ulps)
            && self.o.ulps_eq(&rhs.o, epsilon, max_ulps)
            && self.s.ulps_eq(&rhs.s, epsilon, max_ulps)
    }
}

impl<R> AbsDiffEq for Persp<R>
    where
        R: AbsDiffEq,
        R::Epsilon: Copy,
{
    type Epsilon = R::Epsilon;

    fn default_epsilon() -> R::Epsilon {
        R::default_epsilon()
    }

    fn abs_diff_eq(&self, rhs: &Self, epsilon: R::Epsilon) -> bool {
        self.0.abs_diff_eq(&rhs.0, epsilon)
    }
}

impl<R> RelativeEq for Persp<R>
    where
        R: RelativeEq,
        R::Epsilon: Copy,
{
    fn default_max_relative() -> R::Epsilon {
        R::default_max_relative()
    }

    fn relative_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_relative: R::Epsilon) -> bool {
        self.0.relative_eq(&rhs.0, epsilon, max_relative)
    }
}

impl<R> UlpsEq for Persp<R>
    where
        R: UlpsEq,
        R::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        R::default_max_ulps()
    }

    fn ulps_eq(&self, rhs: &Self, epsilon: R::Epsilon, max_ulps: u32) -> bool {
        self.0.ulps_eq(&rhs.0, epsilon, max_ulps)
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
