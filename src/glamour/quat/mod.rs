use num_traits::{Float, One, Zero};

use crate::glamour::{mat::Mat4, unit::Unit, vec::Vec4};

mod convert;
mod num;
mod ops;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Quat<R> {
    pub w: R,
    pub i: R,
    pub j: R,
    pub k: R,
}

impl<R> Quat<R> {
    pub const fn new(w: R, i: R, j: R, k: R) -> Self {
        Quat { w, i, j, k }
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn with_look_at_lh(fwd: Unit<Vec4<R>>, up: Unit<Vec4<R>>) -> Unit<Quat<R>> {
        Mat4::with_look_at_lh(fwd, up).into()
    }

    pub fn with_axis_angle(axis: Unit<Vec4<R>>, angle: R) -> Unit<Quat<R>> {
        let half = R::one() / (R::one() + R::one());
        let (sin, cos) = R::sin_cos(angle * half);
        Unit::from(Quat::new(cos, axis.x * sin, axis.y * sin, axis.z * sin))
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn c(&self) -> Self {
        Quat::new(self.w, -self.i, -self.j, -self.k)
    }

    pub fn is_nan(&self) -> bool {
        self.w.is_nan() || self.i.is_nan() || self.j.is_nan() || self.k.is_nan()
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn inv(&self) -> Self {
        self.c() / self.abssq()
    }
}

impl<R> Quat<R>
where
    R: Float,
{
    pub fn abssq(&self) -> R {
        self.w.powi(2) + self.i.powi(2) + self.j.powi(2) + self.k.powi(2)
    }
}

impl<R> Quat<R>
where
    R: Zero + One,
{
    pub fn identity() -> Self {
        Quat {
            w: R::one(),
            i: R::zero(),
            j: R::zero(),
            k: R::zero(),
        }
    }
}

impl<R> std::fmt::Display for Quat<R>
where
    R: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + i{} + j{} + k{}", self.w, self.i, self.j, self.k)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use proptest::{
        num::f32::{INFINITE, NEGATIVE, NORMAL, POSITIVE, QUIET_NAN as NAN, SUBNORMAL, ZERO},
        prop_assert, proptest,
    };
    use serde_test::{assert_tokens, Token};

    use super::*;
    use crate::glamour::test_helpers::proptest::{bounded_f32, bounded_nonzero_f32, quat, unit_vec4};

    #[test]
    fn quat_provides_identity_constructor() {
        let q: Quat<f32> = Quat::identity();
        assert_eq!(q.w, 1.0f32);
        assert_eq!(q.i, 0.0f32);
        assert_eq!(q.j, 0.0f32);
        assert_eq!(q.k, 0.0f32);
    }

    #[test]
    fn quat_provides_new_constructor() {
        let q: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q.w, 1.0f32);
        assert_eq!(q.i, 2.0f32);
        assert_eq!(q.j, 3.0f32);
        assert_eq!(q.k, 4.0f32);
    }

    #[test]
    fn quat_implements_abssq_methods() {
        let q = Quat::new(1.0f32, 1.0, 1.0, 1.0);
        assert_eq!(q.abssq(), 4.0f32);
    }

    #[test]
    fn quat_implements_conjugation() {
        let q: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        let c: Quat<f32> = Quat::new(1.0, -2.0, -3.0, -4.0);
        assert_eq!(q.c(), c);
    }

    #[test]
    fn quat_implements_inversion() {
        let q: Quat<f32> = Quat::new(1.0, 2.0, 3.0, 4.0);
        let i: Quat<f32> = Quat::new(1.0, -2.0, -3.0, -4.0) / 30.0;
        assert_eq!(q.inv(), i);
    }

    #[test]
    fn quat_implements_serde() {
        let a: Quat<f32> = Quat::identity();

        assert_tokens(
            &a,
            &[
                Token::Struct { name: "Quat", len: 4 },
                Token::Str("w"),
                Token::F32(1.0f32),
                Token::Str("i"),
                Token::F32(0.0f32),
                Token::Str("j"),
                Token::F32(0.0f32),
                Token::Str("k"),
                Token::F32(0.0f32),
                Token::StructEnd,
            ],
        );
    }

    proptest! {
        // NaN Tests
        #[test]
        fn quat_is_nan_returns_true_for_nan_components(a in quat(NAN)) {
            prop_assert!(a.is_nan())
        }

        #[test]
        fn quat_is_nan_returns_false_for_non_nan_components(a in quat(NORMAL | POSITIVE | NEGATIVE | ZERO | INFINITE | SUBNORMAL)) {
            prop_assert!(!a.is_nan())
        }

        #[test]
        fn quat_with_angle_axis_is_equal_to_nalgebra(angle in bounded_f32(-62, 63), axis in unit_vec4(bounded_nonzero_f32(-62, 63))) {
            let glamour_result = Quat::with_axis_angle(axis, angle);
            let nalgebra_result = nalgebra::UnitQuaternion::from_axis_angle(&axis.into(), angle);

            assert_ulps_eq!(glamour_result, nalgebra_result);
        }

        #[test]
        fn quat_with_angle_axis_is_equal_to_cgmath(angle in bounded_f32(-62, 63), axis in unit_vec4(bounded_nonzero_f32(-62, 63))) {
            use cgmath::Rotation3;

            let glamour_result = Quat::with_axis_angle(axis, angle);
            let cgmath_result = cgmath::Quaternion::from_axis_angle(axis.into(), cgmath::Rad(angle));

            assert_ulps_eq!(glamour_result, cgmath_result);
        }
    }
}
