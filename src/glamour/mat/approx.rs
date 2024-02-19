use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use super::Mat4;

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

#[cfg(test)]
mod tests {
    use approx::{assert_abs_diff_eq, assert_relative_eq, assert_ulps_eq};

    use super::*;

    #[test]
    fn mat4_implements_abs_diff_eq() {
        let a: Mat4<f32> = Mat4::identity();
        let b: Mat4<f32> = Mat4::identity() * 2.0;

        assert_abs_diff_eq!(a * 0.0, b * 0.0);
    }

    #[test]
    fn mat4_implements_relative_eq() {
        let a: Mat4<f32> = Mat4::identity();
        let b: Mat4<f32> = Mat4::identity() * 2.0;

        assert_relative_eq!(a, b / 2.0);
    }

    #[test]
    fn mat4_implements_ulps_eq() {
        let a: Mat4<f32> = Mat4::identity();
        let b: Mat4<f32> = Mat4::identity() * 2.0;

        assert_ulps_eq!(a, b / 2.0);
    }
}
