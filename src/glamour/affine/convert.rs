use crate::glamour::affine::Affine;
use crate::glamour::mat::Mat4;
use num_traits::float::Float;
use num_traits::NumAssign;

impl<R> From<Affine<R>> for Mat4<R>
where
    R: Float + NumAssign,
{
    fn from(v: Affine<R>) -> Self {
        From::from(&v)
    }
}

impl<'a, R> From<&'a Affine<R>> for Mat4<R>
where
    R: Float + NumAssign,
{
    fn from(v: &'a Affine<R>) -> Self {
        let mut m: Mat4<R> = v.o.into();
        m[(0, 0)] *= v.s;
        m[(1, 1)] *= v.s;
        m[(2, 2)] *= v.s;
        m[(0, 3)] = v.t[0];
        m[(1, 3)] = v.t[1];
        m[(2, 3)] = v.t[2];
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glamour::test_helpers::proptest::{affine, bounded_f32};
    use approx::ulps_eq;
    use proptest::{prop_assert, proptest};

    proptest! {
        #[test]
        #[ignore]
        fn from_affine_for_mat_is_equal_to_nalgebra(glamour_lhs in affine(bounded_f32(-62, 63))) {
            let glamour_result: Mat4<f32> = glamour_lhs.into();
            let nalgebra_lhs = nalgebra::Similarity3::from_parts(
                nalgebra::Translation3::new(glamour_lhs.t.x, glamour_lhs.t.y, glamour_lhs.t.z),
                nalgebra::Unit::from_quaternion(nalgebra::Quaternion::new(glamour_lhs.o.w, glamour_lhs.o.i, glamour_lhs.o.j, glamour_lhs.o.k)),
                glamour_lhs.s,
            );
            let nalgebra_result = nalgebra_lhs.to_homogeneous();

            prop_assert!(ulps_eq!(glamour_result, nalgebra_result), "left\t= {glamour_result:?}\nright (transposed)\t= {:?}", nalgebra_result.transpose());
        }

        #[test]
        fn from_affine_for_mat_is_equal_to_cgmath(glamour_lhs in affine(bounded_f32(-62, 63))) {
            let glamour_result: Mat4<f32> = glamour_lhs.into();
            let cgmath_lhs = cgmath::Decomposed {
                disp: cgmath::Vector3::new(glamour_lhs.t.x, glamour_lhs.t.y, glamour_lhs.t.z),
                rot: Into::<cgmath::Quaternion<f32>>::into(glamour_lhs.o.0),
                scale: glamour_lhs.s,
            };
            let cgmath_result: cgmath::Matrix4<f32> = cgmath_lhs.into();

            use cgmath::Matrix;
            prop_assert!(ulps_eq!(glamour_result, cgmath_result), "left\t= {glamour_result:?}\nright (transposed)\t= {:?}", cgmath_result.transpose());
        }
    }
}
