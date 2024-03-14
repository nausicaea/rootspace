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

    use std::sync::OnceLock;

    use approx::{assert_relative_eq, assert_ulps_eq, relative_eq, ulps_eq};
    use proptest::{num::f32::NORMAL, prelude::*};

    use crate::glamour::{num::ToMatrix, quat::tests::{quat, unit_quat}, unit::Unit};

    use super::*;

    fn two_pow_minus_62f32() -> f32 {
        static MIN_POS_F32: OnceLock<f32> = OnceLock::new();
        *MIN_POS_F32.get_or_init(|| (2.0).powi(-62))
    }

    fn two_pow_63f32() -> f32 {
        static MAX_POS_F32: OnceLock<f32> = OnceLock::new();
        *MAX_POS_F32.get_or_init(|| (2.0).powi(63))
    }

    fn minus_two_pow_minus_62f32() -> f32 {
        static MIN_NEG_F32: OnceLock<f32> = OnceLock::new();
        *MIN_NEG_F32.get_or_init(|| (-2.0).powi(-62))
    }

    fn minus_two_pow_63f32() -> f32 {
        static MAX_NEG_F32: OnceLock<f32> = OnceLock::new();
        *MAX_NEG_F32.get_or_init(|| (-2.0).powi(63))
    }

    fn pos_f32_range() -> std::ops::Range<f32> {
        two_pow_minus_62f32()..two_pow_63f32()
    }

    fn neg_f32_range() -> std::ops::Range<f32> {
        minus_two_pow_63f32()..minus_two_pow_minus_62f32()
    }

    fn bounded_f32() -> impl Strategy<Value = f32> {
        proptest::strategy::Union::new([neg_f32_range().boxed(), proptest::num::f32::ZERO.boxed(), pos_f32_range().boxed()])
    }

    fn bounded_nonzero_f32() -> impl Strategy<Value = f32> {
        proptest::strategy::Union::new([neg_f32_range().boxed(), pos_f32_range().boxed()])
    }

    #[test]
    fn quat_implements_mul_for_vec4() {
        let q: Quat<f32> = Quat::identity();
        let v: Vec4<f32> = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(q * v, v);
    }

    proptest! {
        #[test]
        fn bounded_f32_quat_mul_does_not_cause_nans(lhs in quat(bounded_f32()), rhs in quat(bounded_f32())) {
            prop_assert!(!(lhs * rhs).is_nan());
        }

        #[test]
        fn bounded_nonzero_f32_quat_mul_does_not_cause_nans(lhs in quat(bounded_nonzero_f32()), rhs in quat(bounded_nonzero_f32())) {
            prop_assert!(!(lhs * rhs).is_nan());
        }

        #[test]
        fn glamour_quat_mul_is_equal_to_nalgebra(glamour_a in quat(bounded_f32()), glamour_b in quat(bounded_f32())) {
            let nalgebra_a = nalgebra::Quaternion::new(glamour_a.w, glamour_a.i, glamour_a.j, glamour_a.k);
            let nalgebra_b = nalgebra::Quaternion::new(glamour_b.w, glamour_b.i, glamour_b.j, glamour_b.k);

            prop_assert!(ulps_eq!(glamour_b * glamour_a, nalgebra_b * nalgebra_a));
        }

        #[test]
        fn glamour_quat_mul_is_equal_to_reordered_cgmath(glamour_a in quat(bounded_f32()), glamour_b in quat(bounded_f32())) {
            let cgmath_a = cgmath::Quaternion::new(glamour_a.w, glamour_a.i, glamour_a.j, glamour_a.k);
            let cgmath_b = cgmath::Quaternion::new(glamour_b.w, glamour_b.i, glamour_b.j, glamour_b.k);
            let cgmath_result = cgmath::Quaternion::new(
                cgmath_b.s * cgmath_a.s - cgmath_b.v.x * cgmath_a.v.x - cgmath_b.v.y * cgmath_a.v.y - cgmath_b.v.z * cgmath_a.v.z,
                cgmath_b.s * cgmath_a.v.x + cgmath_b.v.x * cgmath_a.s + cgmath_b.v.y * cgmath_a.v.z - cgmath_b.v.z * cgmath_a.v.y,
                cgmath_b.s * cgmath_a.v.y - cgmath_b.v.x * cgmath_a.v.z + cgmath_b.v.y * cgmath_a.s + cgmath_b.v.z * cgmath_a.v.x,
                cgmath_b.s * cgmath_a.v.z + cgmath_b.v.x * cgmath_a.v.y - cgmath_b.v.y * cgmath_a.v.x + cgmath_b.v.z * cgmath_a.s,
            );
            prop_assert!(ulps_eq!(glamour_b * glamour_a, cgmath_result));
        }

        /// The result of the cgmath-based quaternion multiplication will be different from glamour and nalgebra because the ordering of operands for the j and k components is different, causing different float rounding errors. Therefore, there is an additional test with cgmath that involves manually calculating the product with adjusted operand ordering.
        #[test]
        #[should_panic]
        fn glamour_quat_mul_is_not_equal_to_cgmath(glamour_a in quat(bounded_f32()), glamour_b in quat(bounded_f32())) {
            let cgmath_a = cgmath::Quaternion::new(glamour_a.w, glamour_a.i, glamour_a.j, glamour_a.k);
            let cgmath_b = cgmath::Quaternion::new(glamour_b.w, glamour_b.i, glamour_b.j, glamour_b.k);
            prop_assert!(ulps_eq!(glamour_b * glamour_a, cgmath_b * cgmath_a));
        }

        #[test]
        fn unit_quat_mul_is_the_same_as_rot_mat_mul(glamour_a in unit_quat(bounded_nonzero_f32()), glamour_b in unit_quat(bounded_nonzero_f32())) {
            let qp = glamour_a * glamour_b;
            let mp = Into::<Unit<_>>::into(Into::<Quat<f32>>::into(glamour_a.to_matrix() * glamour_b.to_matrix()));
            prop_assert!(relative_eq!(qp, mp));
        }
    }
}
