use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::vec::Vec4;

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

