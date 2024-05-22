use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::quat::Quat;

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
