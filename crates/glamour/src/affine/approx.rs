use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::affine::Affine;

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

