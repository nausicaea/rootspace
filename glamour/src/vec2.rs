impl_vector!(Vec2, 2, [x, y]);

impl<'a, N> crate::cross::Cross for &'a Vec2<N>
where
    N: num_traits::Num + Copy,
{
    type Output = N;

    fn cross(self, rhs: Self) -> Self::Output {
        self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross::Cross;
    use crate::dot::Dot;
    use approx::assert_ulps_eq;
    use std::ops::{Add, Div, Mul, Sub};

    #[test]
    fn vec2_requires_two_constructor_arguments_and_has_two_components() {
        let v: Vec2<f32> = Vec2::new(0.0f32, 1.0f32);

        assert_eq!(v.0[0], 0.0f32);
        assert_eq!(v.0[1], 1.0f32);
    }

    #[test]
    fn vec2_supports_addition() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = Vec2::new(3.0f32, 4.0f32);

        let c: Vec2<f32> = a.add(&b);
        assert_ulps_eq!(c.0[0], 4.0f32);
        assert_ulps_eq!(c.0[1], 6.0f32);
    }

    #[test]
    fn vec2_supports_subtraction() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = Vec2::new(3.0f32, 4.0f32);

        let c: Vec2<f32> = a.sub(&b);
        assert_ulps_eq!(c.0[0], -2.0f32);
        assert_ulps_eq!(c.0[1], -2.0f32);
    }

    #[test]
    fn vec2_supports_scalar_addition() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = 2.0f32;

        let c: Vec2<f32> = a.add(b);
        assert_ulps_eq!(c.0[0], 3.0f32);
        assert_ulps_eq!(c.0[1], 4.0f32);
    }

    #[test]
    fn vec2_supports_scalar_subtraction() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = 2.0f32;

        let c: Vec2<f32> = a.sub(b);
        assert_ulps_eq!(c.0[0], -1.0f32);
        assert_ulps_eq!(c.0[1], 0.0f32);
    }

    #[test]
    fn vec2_supports_scalar_multiplication() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = 2.0f32;

        let c: Vec2<f32> = a.mul(b);
        assert_ulps_eq!(c.0[0], 2.0f32);
        assert_ulps_eq!(c.0[1], 4.0f32);
    }

    #[test]
    fn vec2_supports_scalar_division() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = 2.0f32;

        let c: Vec2<f32> = a.div(b);
        assert_ulps_eq!(c.0[0], 0.5f32);
        assert_ulps_eq!(c.0[1], 1.0f32);
    }

    #[test]
    fn vec2_supports_dot_product() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = Vec2::new(3.0f32, 4.0f32);

        let c: f32 = a.dot(b);
        assert_ulps_eq!(c, 11.0f32);
    }

    #[test]
    fn vec2_supports_cross_product() {
        let a = Vec2::new(1.0f32, 2.0f32);
        let b = Vec2::new(3.0f32, 4.0f32);

        let c: f32 = a.cross(&b);
        assert_ulps_eq!(c, -2.0f32);
    }

    #[test]
    fn vec2_supports_f64() {
        let _: Vec2<f64> = Vec2::new(1.0f64, 2.0f64);
    }
}
