use num_traits::Float;

use crate::glamour::{mat::Mat4, ops::norm::Norm, vec::Vec4};
use crate::glamour::unit::Unit;

use super::Quat;

impl<R> From<Mat4<R>> for Quat<R>
where
    R: Float,
{
    fn from(v: Mat4<R>) -> Self {
        let half: R = R::one() / (R::one() + R::one());

        if v[(2, 2)] < v[(0, 0)] {
            if v[(0, 0)] > v[(1, 1)] {
                let t = R::one() + v[(0, 0)] - v[(1, 1)] - v[(2, 2)];
                Quat::new(v[(1, 2)] - v[(2, 1)], t, v[(0, 1)] + v[(1, 0)], v[(2, 0)] + v[(0, 2)]) * (half / t.sqrt())
            } else {
                let t = R::one() - v[(0, 0)] + v[(1, 1)] - v[(2, 2)];
                Quat::new(v[(2, 0)] - v[(0, 2)], v[(0, 1)] + v[(1, 0)], t, v[(1, 2)] + v[(2, 1)]) * (half / t.sqrt())
            }
        } else if v[(0, 0)] < -v[(1, 1)] {
            let t = R::one() - v[(0, 0)] - v[(1, 1)] + v[(2, 2)];
            Quat::new(v[(0, 1)] - v[(1, 0)], v[(2, 0)] + v[(0, 2)], v[(1, 2)] + v[(2, 1)], t) * (half / t.sqrt())
        } else {
            let t = R::one() + v[(0, 0)] + v[(1, 1)] + v[(2, 2)];
            Quat::new(t, v[(1, 2)] - v[(2, 1)], v[(2, 0)] - v[(0, 2)], v[(0, 1)] - v[(1, 0)]) * (half / t.sqrt())
        }
    }
}

impl<R> From<Quat<R>> for Mat4<R>
where
    R: Float,
{
    /// Based on information from the [Euclidean Space Blog](https://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToMatrix/index.htm)
    fn from(v: Quat<R>) -> Self {
        let v = Unit::from(v);
        let w = v.w;
        let i = v.i;
        let j = v.j;
        let k = v.k;

        let z = R::zero();
        let o = R::one();
        let t = o + o;

        Mat4::from([[
            o - t * j * j - t * k * k,
            t * i * j - t * k * w,
            t * i * k + t * j * w,
            z],
            [t * i * j + t * k * w,
            o - t * i * i - t * k * k,
            t * j * k - t * i * w,
            z],
            [t * i * k - t * j * w,
            t * j * k + t * i * w,
            o - t * i * i - t * j * j,
            z],
            [z,
            z,
            z,
            o],
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
    use proptest::num::f32::{NEGATIVE, NORMAL, POSITIVE, SUBNORMAL, ZERO};
    use proptest::{prop_assert_eq, proptest};
    use crate::glamour::test_helpers::{bounded_nonzero_f32, quat, vec4};
    use super::*;

    #[test]
    fn quat_implements_from_mat4() {
        let a: Mat4<f32> = Mat4::identity();
        assert_eq!(Quat::<f32>::from(a), Quat::<f32>::identity());
    }

    #[test]
    fn mat4_from_quat_results_in_nan_for_zero_norm() {
        let q = Quat::new(0.0f32, 0.0, 0.0, 0.0);
        let m = Mat4::from(q);
        assert!(m.is_nan());
    }

    proptest! {
        #[test]
        fn from_quat_for_mat_is_equal_to_nalgebra(glamour_lhs in quat(bounded_nonzero_f32(-62, 63))) {
            let nalgebra_lhs = nalgebra::Quaternion::new(glamour_lhs.w, glamour_lhs.i, glamour_lhs.j, glamour_lhs.k);
            let nalgebra_result = Into::<nalgebra::Matrix4<f32>>::into(nalgebra_lhs);

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
