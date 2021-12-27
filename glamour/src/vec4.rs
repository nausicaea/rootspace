impl_vector!(Vec4, 4, [x, y, z, w]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dot::Dot;
    use approx::assert_ulps_eq;
    use std::ops::{Add, Div, Mul, Sub};

    #[test]
    fn vec4_requires_four_constructor_arguments_and_has_four_components() {
        let v: Vec4<f32> = Vec4::new(0.0f32, 1.0f32, 2.0f32, 3.0f32);

        assert_eq!(v.0[0], 0.0f32);
        assert_eq!(v.0[1], 1.0f32);
        assert_eq!(v.0[2], 2.0f32);
        assert_eq!(v.0[3], 3.0f32);
    }

    #[test]
    fn vec4_supports_addition() {
        let a = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
        let b = Vec4::new(3.0f32, 4.0f32, 3.0f32, 5.0f32);

        let c: Vec4<f32> = a.add(&b);
        assert_ulps_eq!(c.0[0], 4.0f32);
        assert_ulps_eq!(c.0[1], 6.0f32);
        assert_ulps_eq!(c.0[2], 6.0f32);
        assert_ulps_eq!(c.0[3], 9.0f32);
    }

    #[test]
    fn vec4_supports_subtraction() {
        let a = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
        let b = Vec4::new(3.0f32, 4.0f32, 3.0f32, 5.0f32);

        let c: Vec4<f32> = a.sub(&b);
        assert_ulps_eq!(c.0[0], -2.0f32);
        assert_ulps_eq!(c.0[1], -2.0f32);
        assert_ulps_eq!(c.0[2], 0.0f32);
        assert_ulps_eq!(c.0[3], -1.0f32);
    }

    #[test]
    fn vec4_supports_scalar_addition() {
        let a = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
        let b = 2.0f32;

        let c: Vec4<f32> = a.add(b);
        assert_ulps_eq!(c.0[0], 3.0f32);
        assert_ulps_eq!(c.0[1], 4.0f32);
        assert_ulps_eq!(c.0[2], 5.0f32);
        assert_ulps_eq!(c.0[3], 6.0f32);
    }

    #[test]
    fn vec4_supports_scalar_subtraction() {
        let a = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
        let b = 2.0f32;

        let c: Vec4<f32> = a.sub(b);
        assert_ulps_eq!(c.0[0], -1.0f32);
        assert_ulps_eq!(c.0[1], 0.0f32);
        assert_ulps_eq!(c.0[2], 1.0f32);
        assert_ulps_eq!(c.0[3], 2.0f32);
    }

    #[test]
    fn vec4_supports_scalar_multiplication() {
        let a = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
        let b = 2.0f32;

        let c: Vec4<f32> = a.mul(b);
        assert_ulps_eq!(c.0[0], 2.0f32);
        assert_ulps_eq!(c.0[1], 4.0f32);
        assert_ulps_eq!(c.0[2], 6.0f32);
        assert_ulps_eq!(c.0[3], 8.0f32);
    }

    #[test]
    fn vec4_supports_scalar_division() {
        let a = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
        let b = 2.0f32;

        let c: Vec4<f32> = a.div(b);
        assert_ulps_eq!(c.0[0], 0.5f32);
        assert_ulps_eq!(c.0[1], 1.0f32);
        assert_ulps_eq!(c.0[2], 1.5f32);
        assert_ulps_eq!(c.0[3], 2.0f32);
    }

    #[test]
    fn vec4_supports_dot_product() {
        let a = Vec4::new(1.0f32, 2.0f32, 3.0f32, 4.0f32);
        let b = Vec4::new(5.0f32, 6.0f32, 7.0f32, 8.0f32);

        let c: f32 = a.dot(b);
        assert_ulps_eq!(c, 70.0f32);
    }

    #[test]
    fn vec4_supports_f64() {
        let _: Vec4<f64> = Vec4::new(1.0f64, 2.0f64, 3.0f64, 4.0f64);
    }
}
