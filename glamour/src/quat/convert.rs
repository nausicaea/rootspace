use num_traits::Float;

use super::Quat;
use crate::{mat::Mat4, unit::Unit, vec::Vec4};

impl<R> From<Mat4<R>> for Unit<Quat<R>>
where
    R: Float,
{
    /// Based on information from [Euclidean Space Blog](https://www.euclideanspace.com/maths/geometry/rotations/conversions/matrixToQuaternion/index.htm)
    fn from(v: Mat4<R>) -> Self {
        let one = R::one();
        let two = one + one;
        let one_quarter = one / (two + two);
        let v00 = v[(0, 0)];
        let v11 = v[(1, 1)];
        let v22 = v[(2, 2)];
        let trace = v00 + v11 + v22;

        let q = if trace > R::zero() {
            let s = (one + trace).sqrt() * two;
            Quat::new(
                one_quarter * s,
                (v[(2, 1)] - v[(1, 2)]) / s,
                (v[(0, 2)] - v[(2, 0)]) / s,
                (v[(1, 0)] - v[(0, 1)]) / s,
            )
        } else if v00 > v11 && v00 > v22 {
            let s = (one + v00 - v11 - v22).sqrt() * two;
            Quat::new(
                (v[(2, 1)] - v[(1, 2)]) / s,
                one_quarter * s,
                (v[(0, 1)] + v[(1, 0)]) / s,
                (v[(0, 2)] + v[(2, 0)]) / s,
            )
        } else if v11 > v22 {
            let s = (one + v11 - v00 - v22).sqrt() * two;
            Quat::new(
                (v[(0, 2)] - v[(2, 0)]) / s,
                (v[(0, 1)] + v[(1, 0)]) / s,
                one_quarter * s,
                (v[(1, 2)] + v[(2, 1)]) / s,
            )
        } else {
            let s = (one + v22 - v00 - v11).sqrt() * two;
            Quat::new(
                (v[(1, 0)] - v[(0, 1)]) / s,
                (v[(0, 2)] + v[(2, 0)]) / s,
                (v[(1, 2)] + v[(2, 1)]) / s,
                one_quarter * s,
            )
        };

        Unit::from(q)
    }
}

impl<R> From<Unit<Quat<R>>> for Mat4<R>
where
    R: Float,
{
    /// Based on information from the [Euclidean Space Blog](https://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToMatrix/index.htm)
    fn from(v: Unit<Quat<R>>) -> Self {
        let w = v.w;
        let i = v.i;
        let j = v.j;
        let k = v.k;

        let z = R::zero();
        let o = R::one();
        let t = o + o;

        Mat4::from([
            [
                o - t * j * j - t * k * k,
                t * i * j - t * k * w,
                t * i * k + t * j * w,
                z,
            ],
            [
                t * i * j + t * k * w,
                o - t * i * i - t * k * k,
                t * j * k - t * i * w,
                z,
            ],
            [
                t * i * k - t * j * w,
                t * j * k + t * i * w,
                o - t * i * i - t * j * j,
                z,
            ],
            [z, z, z, o],
        ])
    }
}

impl<R> From<Vec4<R>> for Quat<R>
where
    R: num_traits::Zero,
{
    fn from(value: Vec4<R>) -> Self {
        Quat::new(value.w, value.x, value.y, value.z)
    }
}

impl<R> From<Quat<R>> for Vec4<R>
where
    R: num_traits::Zero,
{
    fn from(value: Quat<R>) -> Self {
        Vec4::new(value.i, value.j, value.k, value.w)
    }
}

#[cfg(test)]
mod tests {
    use approx::{relative_eq, ulps_eq};
    use cgmath::InnerSpace;
    use proptest::{
        num::f32::{NEGATIVE, NORMAL, POSITIVE, SUBNORMAL, ZERO},
        prop_assert, prop_assert_eq, proptest,
    };

    use super::*;
    use crate::test_helpers::proptest::{bounded_nonzero_f32, mat4, quat, unit_quat, vec4};

    #[test]
    fn quat_implements_from_mat4() {
        let a: Unit<Quat<f32>> = Mat4::identity().into();
        assert_eq!(a, Unit::from(Quat::<f32>::identity()));
    }

    #[test]
    fn mat4_from_quat_results_in_nan_for_zero_norm() {
        let q = Unit::from(Quat::new(0.0f32, 0.0, 0.0, 0.0));
        let m = Mat4::from(q);
        assert!(m.is_nan());
    }

    proptest! {
        /// Nalgebra uses an iterative algorithm to calculate the quaternion from a matrix, but the algorithm doesn't exit even with a given max_iteration count due to using a potentially unbounded loop internally. We cannot use this test as is.
        #[test]
        #[ignore]
        fn from_mat_for_quat_is_equal_to_nalgebra(glamour_lhs in mat4(bounded_nonzero_f32(-62, 63))) {
            let glamour_result = Into::<Unit<Quat<f32>>>::into(glamour_lhs);
            let nalgebra_lhs = nalgebra::Matrix3::new(
                glamour_lhs[(0, 0)], glamour_lhs[(1, 0)], glamour_lhs[(2, 0)],
                glamour_lhs[(0, 1)], glamour_lhs[(1, 1)], glamour_lhs[(2, 1)],
                glamour_lhs[(0, 2)], glamour_lhs[(1, 2)], glamour_lhs[(2, 2)],
            );
            let nalgebra_result: nalgebra::UnitQuaternion<f32> = nalgebra::UnitQuaternion::from_matrix_eps(&nalgebra_lhs, 10.0 * f32::EPSILON, 1, nalgebra::UnitQuaternion::identity());

            prop_assert!(ulps_eq!(glamour_result.0, *nalgebra_result.quaternion()));
        }

        #[test]
        fn from_mat_for_quat_is_equal_to_cgmath(glamour_lhs in mat4(bounded_nonzero_f32(-62, 63))) {
            let glamour_result = Into::<Unit<Quat<f32>>>::into(glamour_lhs).0;
            let cgmath_lhs = cgmath::Matrix3::new(
                glamour_lhs[(0, 0)], glamour_lhs[(1, 0)], glamour_lhs[(2, 0)],
                glamour_lhs[(0, 1)], glamour_lhs[(1, 1)], glamour_lhs[(2, 1)],
                glamour_lhs[(0, 2)], glamour_lhs[(1, 2)], glamour_lhs[(2, 2)],
            );
            let cgmath_result = Into::<cgmath::Quaternion<f32>>::into(cgmath_lhs).normalize();

            prop_assert!(ulps_eq!(glamour_result, cgmath_result));
        }

        /// Nalgebra likely uses a different conversion algorithm which causes large rounding errors
        #[test]
        #[should_panic]
        fn from_quat_for_mat_is_equal_to_nalgebra(glamour_lhs in unit_quat(bounded_nonzero_f32(-62, 63))) {
            let glamour_result = Into::<Mat4<f32>>::into(glamour_lhs);
            let nalgebra_lhs: nalgebra::UnitQuaternion<f32> = glamour_lhs.into();
            let nalgebra_result = Into::<nalgebra::Matrix4<f32>>::into(nalgebra_lhs);

            prop_assert!(ulps_eq!(glamour_result, nalgebra_result), "left: {glamour_result:?}\nright: {nalgebra_result:?}");
        }

        /// Nalgebra likely uses a different conversion algorithm which causes large rounding errors
        #[test]
        fn from_quat_for_mat_has_large_rounding_differences_to_nalgebra(glamour_lhs in unit_quat(bounded_nonzero_f32(-62, 63))) {
            let glamour_result = Into::<Mat4<f32>>::into(glamour_lhs);
            let nalgebra_lhs: nalgebra::UnitQuaternion<f32> = glamour_lhs.into();
            let nalgebra_result = Into::<nalgebra::Matrix4<f32>>::into(nalgebra_lhs);

            prop_assert!(relative_eq!(glamour_result, nalgebra_result, max_relative = 1e-2), "left: {glamour_result:?}\nright: {nalgebra_result:?}");
        }

        #[test]
        fn from_quat_for_mat_is_equal_to_cgmath(glamour_lhs in unit_quat(bounded_nonzero_f32(-62, 63))) {
            let glamour_result = Into::<Mat4<f32>>::into(glamour_lhs);
            let cgmath_lhs: cgmath::Quaternion<f32> = glamour_lhs.into();
            let cgmath_result = Into::<cgmath::Matrix4<f32>>::into(cgmath_lhs);

            prop_assert!(ulps_eq!(glamour_result, cgmath_result), "left: {glamour_result:?}\nright: {cgmath_result:?}");
        }

        #[test]
        fn from_vec_for_quat_moves_w_to_the_front(lhs in vec4(NORMAL | SUBNORMAL | POSITIVE | NEGATIVE | ZERO)) {
            let result = Quat::new(lhs.w, lhs.x, lhs.y, lhs.z);
            prop_assert_eq!(Into::<Quat<f32>>::into(lhs), result);
        }

        #[test]
        fn from_quat_for_vec_moves_w_to_the_back(lhs in quat(NORMAL | SUBNORMAL | POSITIVE | NEGATIVE | ZERO)) {
            let result = Vec4::new(lhs.i, lhs.j, lhs.k, lhs.w);
            prop_assert_eq!(Into::<Vec4<f32>>::into(lhs), result);
        }
    }
}
