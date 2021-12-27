impl_vector!(Vec3, 3, [x, y, z]);

impl<'a, N> crate::cross::Cross for &'a Vec3<N>
where
    N: num_traits::Num + Copy,
{
    type Output = Vec3<N>;

    fn cross(self, rhs: Self) -> Self::Output {
        Vec3([
            self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
            self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
        ])
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
    fn vec3_requires_three_constructor_arguments_and_has_three_components() {
        let v: Vec3<f32> = Vec3::new(0.0f32, 1.0f32, 2.0f32);

        assert_eq!(v.0[0], 0.0f32);
        assert_eq!(v.0[1], 1.0f32);
        assert_eq!(v.0[2], 2.0f32);
    }

    #[test]
    fn vec3_supports_addition() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = Vec3::new(3.0f32, 4.0f32, 3.0f32);

        let c: Vec3<f32> = a.add(&b);
        assert_ulps_eq!(c.0[0], 4.0f32);
        assert_ulps_eq!(c.0[1], 6.0f32);
        assert_ulps_eq!(c.0[2], 6.0f32);
    }

    #[test]
    fn vec3_supports_subtraction() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = Vec3::new(3.0f32, 4.0f32, 3.0f32);

        let c: Vec3<f32> = a.sub(&b);
        assert_ulps_eq!(c.0[0], -2.0f32);
        assert_ulps_eq!(c.0[1], -2.0f32);
        assert_ulps_eq!(c.0[2], 0.0f32);
    }

    #[test]
    fn vec3_supports_scalar_addition() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Vec3<f32> = a.add(b);
        assert_ulps_eq!(c.0[0], 3.0f32);
        assert_ulps_eq!(c.0[1], 4.0f32);
        assert_ulps_eq!(c.0[2], 5.0f32);
    }

    #[test]
    fn vec3_supports_scalar_subtraction() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Vec3<f32> = a.sub(b);
        assert_ulps_eq!(c.0[0], -1.0f32);
        assert_ulps_eq!(c.0[1], 0.0f32);
        assert_ulps_eq!(c.0[2], 1.0f32);
    }

    #[test]
    fn vec3_supports_scalar_multiplication() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Vec3<f32> = a.mul(b);
        assert_ulps_eq!(c.0[0], 2.0f32);
        assert_ulps_eq!(c.0[1], 4.0f32);
        assert_ulps_eq!(c.0[2], 6.0f32);
    }

    #[test]
    fn vec3_supports_scalar_division() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = 2.0f32;

        let c: Vec3<f32> = a.div(b);
        assert_ulps_eq!(c.0[0], 0.5f32);
        assert_ulps_eq!(c.0[1], 1.0f32);
        assert_ulps_eq!(c.0[2], 1.5f32);
    }

    #[test]
    fn vec3_supports_dot_product() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = Vec3::new(4.0f32, 5.0f32, 6.0f32);

        let c: f32 = a.dot(b);
        assert_ulps_eq!(c, 32.0f32);
    }

    #[test]
    fn vec3_supports_cross_product() {
        let a = Vec3::new(1.0f32, 2.0f32, 3.0f32);
        let b = Vec3::new(4.0f32, 5.0f32, 6.0f32);

        let c: Vec3<f32> = a.cross(&b);
        assert_ulps_eq!(c.0[0], -3.0f32);
        assert_ulps_eq!(c.0[1], -3.0f32);
        assert_ulps_eq!(c.0[2], 6.0f32);
    }

    #[test]
    fn vec3_supports_f64() {
        let _: Vec3<f64> = Vec3::new(1.0f64, 2.0f64, 3.0f64);
    }
}
