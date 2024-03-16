use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use super::Vec4;

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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use approx::{assert_abs_diff_eq, assert_relative_eq, assert_ulps_eq};

    use super::*;
    use crate::glamour::num::One;

    #[test]
    fn mat4_implements_abs_diff_eq() {
        let a: Vec4<f32> = Vec4::one();
        let b: Vec4<f32> = Vec4::one() * 2.0;

        assert_abs_diff_eq!(a * 0.0, b * 0.0);
    }

    #[test]
    fn mat4_implements_relative_eq() {
        let a: Vec4<f32> = Vec4::one();
        let b: Vec4<f32> = Vec4::one() * 2.0;

        assert_relative_eq!(a, b / 2.0);
    }

    #[test]
    fn mat4_implements_ulps_eq() {
        let a: Vec4<f32> = Vec4::one();
        let b: Vec4<f32> = Vec4::one() * 2.0;

        assert_ulps_eq!(a, b / 2.0);
    }
}
