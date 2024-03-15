use std::ops::Mul;

use num_traits::Float;

use crate::{
    forward_ref_binop,
    glamour::quat::Quat,
};

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
    use approx::ulps_eq;
    use proptest::{prop_assert, proptest};
    use crate::glamour::{test_helpers::{bounded_f32, bounded_nonzero_f32, quat, vec4}};

    use super::*;

    proptest! {
        #[test]
        fn bounded_f32_quat_mul_does_not_cause_nans(lhs in quat(bounded_f32(-62, 63)), rhs in quat(bounded_f32(-62, 63))) {
            prop_assert!(!(lhs * rhs).is_nan());
        }

        #[test]
        fn bounded_nonzero_f32_quat_mul_does_not_cause_nans(lhs in quat(bounded_nonzero_f32(-62, 63)), rhs in quat(bounded_nonzero_f32(-62, 63))) {
            prop_assert!(!(lhs * rhs).is_nan());
        }

        #[test]
        fn glamour_quat_vec_mul_is_equal_to_nalgebra(glamour_lhs in quat(bounded_f32(-32, 32)), glamour_rhs in vec4(bounded_f32(-32, 32))) {
            let glamour_result: Quat<f32> = glamour_lhs * Into::<Quat<f32>>::into(glamour_rhs) * glamour_lhs.c();
            let nalgebra_lhs = nalgebra::Quaternion::new(glamour_lhs.w, glamour_lhs.i, glamour_lhs.j, glamour_lhs.k);
            let nalgebra_rhs = nalgebra::Vector4::new(glamour_rhs.x, glamour_rhs.y, glamour_rhs.z, glamour_rhs.w);
            let nalgebra_result = nalgebra_lhs * Into::<nalgebra::Quaternion<f32>>::into(nalgebra_rhs) * nalgebra_lhs.conjugate();
            prop_assert!(ulps_eq!(glamour_result, nalgebra_result));
        }

        #[test]
        fn glamour_quat_mul_is_equal_to_nalgebra(glamour_a in quat(bounded_f32(-62, 63)), glamour_b in quat(bounded_f32(-62, 63))) {
            let nalgebra_a = nalgebra::Quaternion::new(glamour_a.w, glamour_a.i, glamour_a.j, glamour_a.k);
            let nalgebra_b = nalgebra::Quaternion::new(glamour_b.w, glamour_b.i, glamour_b.j, glamour_b.k);

            prop_assert!(ulps_eq!(glamour_b * glamour_a, nalgebra_b * nalgebra_a));
        }

        #[test]
        fn glamour_quat_mul_is_equal_to_reordered_cgmath(glamour_a in quat(bounded_f32(-62, 63)), glamour_b in quat(bounded_f32(-62, 63))) {
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
        fn glamour_quat_mul_is_not_equal_to_cgmath(glamour_a in quat(bounded_f32(-62, 63)), glamour_b in quat(bounded_f32(-62, 63))) {
            let cgmath_a = cgmath::Quaternion::new(glamour_a.w, glamour_a.i, glamour_a.j, glamour_a.k);
            let cgmath_b = cgmath::Quaternion::new(glamour_b.w, glamour_b.i, glamour_b.j, glamour_b.k);
            prop_assert!(ulps_eq!(glamour_b * glamour_a, cgmath_b * cgmath_a));
        }
    }
}
