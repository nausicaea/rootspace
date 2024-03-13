use std::ops::Mul;

use num_traits::Float;

use crate::{
    forward_ref_binop,
    glamour::{quat::Quat, vec::Vec4},
};

impl<'a, 'b, R> Mul<&'b Vec4<R>> for &'a Quat<R>
where
    R: Float,
{
    type Output = Vec4<R>;

    fn mul(self, rhs: &'b Vec4<R>) -> Self::Output {
        let q = Quat::new(R::zero(), rhs.x, rhs.y, rhs.z);
        let rhs_1 = self * q * self.c();
        Vec4::new(rhs_1.i, rhs_1.j, rhs_1.k, rhs.w)
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Quat<R>, Vec4<R>, Vec4<R>);

impl<'a, 'b, R> Mul<&'b Quat<R>> for &'a Quat<R>
where
    R: Float,
{
    type Output = Quat<R>;

    fn mul(self, rhs: &'b Quat<R>) -> Self::Output {
        let w1 = self.w;
        let i1 = self.i;
        let j1 = self.j;
        let k1 = self.k;
        let w2 = rhs.w;
        let i2 = rhs.i;
        let j2 = rhs.j;
        let k2 = rhs.k;

        Quat::new(
             w1 * w2 - i1 * i2 - j1 * j2 - k1 * k2,
             w1 * i2 + i1 * w2 + j1 * k2 - k1 * j2,
             w1 * j2 - i1 * k2 + j1 * w2 + k1 * i2,
             w1 * k2 + i1 * j2 - j1 * i2 + k1 * w2,
        )
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Quat<R>, Quat<R>, Quat<R>);

#[cfg(test)]
mod tests {

    use approx::relative_eq;
    use proptest::{collection::vec, num::f32::NORMAL, prelude::*};

    use crate::glamour::{num::ToMatrix, quat::tests::{quat, unit_quat}, unit::Unit};

    use super::*;

    #[test]
    fn quat_implements_mul_for_vec4() {
        let q: Quat<f32> = Quat::identity();
        let v: Vec4<f32> = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q * v, v);
    }

    #[test]
    fn quat_implements_mul_for_quat() {
        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [5.0f32, 6.0, 7.0, 8.0];
        let expected = Quat::new(-60.0, 20.0, 14.0, 32.0);

        let glamour_a = Quat::new(a[0], a[1], a[2], a[3]);
        let glamour_b = Quat::new(b[0], b[1], b[2], b[3]);
        let glamour_result = glamour_b * glamour_a;

        let nalgebra_a = nalgebra::Quaternion::new(a[0], a[1], a[2], a[3]);
        let nalgebra_b = nalgebra::Quaternion::new(b[0], b[1], b[2], b[3]);
        let nalgebra_result = nalgebra_b * nalgebra_a;

        let cgmath_a = cgmath::Quaternion::new(a[0], a[1], a[2], a[3]);
        let cgmath_b = cgmath::Quaternion::new(b[0], b[1], b[2], b[3]);
        let cgmath_result = cgmath_b * cgmath_a;

        assert_eq!(expected, glamour_result, "glamour comparison");
        assert_eq!(expected, nalgebra_result, "nalgebra comparison");
        assert_eq!(expected, cgmath_result, "cgmath comparison");
    }

    const MIN_F32: f32 = (2.0).powi(-62);
    const MAX_F32: f32 = (2.0).powi(63);

    proptest! {
        #[test]
        fn f32_mul_behavior(a in NORMAL, b in NORMAL) {
            let result = b * a;
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn f32_div_behavior(a in NORMAL, b in NORMAL) {
            let result = b / a;
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn f32_add_behavior(a in NORMAL, b in NORMAL) {
            let result = b + a;
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn f32_sub_behavior(a in NORMAL, b in NORMAL) {
            let result = b - a;
            prop_assert!(!result.is_nan());
        }

        #[test]
        //fn nan_test(a in quat(NORMAL), b in quat(NORMAL)) {
        fn nan_test(a in quat(MIN_F32..MAX_F32), b in quat(MIN_F32..MAX_F32)) {
            let result = b * a;
            if result.is_nan() {
                dbg!(&a);
                dbg!(&b);
                dbg!(&result);
                dbg!(b.w * a.k + b.i * a.j - b.j * a.i + b.k * a.w);
                dbg!(b.w * a.k);
                dbg!(b.i * a.j);
                dbg!(- b.j * a.i);
                dbg!(b.k * a.w);
                // sis is se probleme
                dbg!(f32::INFINITY + f32::NEG_INFINITY);
                dbg!(f32::INFINITY - f32::INFINITY);
            }
            prop_assert!(!result.is_nan());
        }

        #[test]
        fn quat_mul_with_nalgebra_and_cgmath(glamour_a in quat(NORMAL), glamour_b in quat(NORMAL)) {
            let glamour_result = glamour_b * glamour_a;

            let nalgebra_a = nalgebra::Quaternion::new(glamour_a.w, glamour_a.i, glamour_a.j, glamour_a.k);
            let nalgebra_b = nalgebra::Quaternion::new(glamour_b.w, glamour_b.i, glamour_b.j, glamour_b.k);
            let nalgebra_result = nalgebra_b * nalgebra_a;

            let cgmath_a = cgmath::Quaternion::new(glamour_a.w, glamour_a.i, glamour_a.j, glamour_a.k);
            let cgmath_b = cgmath::Quaternion::new(glamour_b.w, glamour_b.i, glamour_b.j, glamour_b.k);
            let cgmath_result = cgmath_b * cgmath_a;

            prop_assume!(!nalgebra_is_nan(&nalgebra_result));

            prop_assert!(relative_eq!(glamour_result, nalgebra_result), "nalgebra comparison");
            prop_assert!(relative_eq!(glamour_result, cgmath_result), "cgmath comparison");
        }

        #[test]
        fn unit_quat_mul_is_the_same_as_rot_mat_mul(a in unit_quat(NORMAL), b in unit_quat(NORMAL)) {
            let qp = b * a;
            let mp = Into::<Unit<_>>::into(Into::<Quat<f32>>::into(b.to_matrix() * a.to_matrix()));
            prop_assert!(relative_eq!(qp, mp));
        }
    }

    pub fn nalgebra_is_nan(input: &nalgebra::Quaternion<f32>) -> bool {
        input.i.is_nan() || input.j.is_nan() || input.k.is_nan() || input.w.is_nan()
    }
}
