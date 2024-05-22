use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::persp::Persp;

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
