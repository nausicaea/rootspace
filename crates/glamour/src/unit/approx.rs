use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::unit::Unit;

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
