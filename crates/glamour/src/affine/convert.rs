use num_traits::{NumAssign, float::Float};

use crate::{affine::Affine, mat::Mat4};

impl<R> From<Affine<R>> for Mat4<R>
where
    R: Float,
{
    fn from(v: Affine<R>) -> Self {
        From::from(&v)
    }
}

impl<'a, R> From<&'a Affine<R>> for Mat4<R>
where
    R: Float,
{
    fn from(v: &'a Affine<R>) -> Self {
        let mut m: Mat4<R> = v.o.into();
        m = m * v.s;
        m[(0, 3)] = v.t[0];
        m[(1, 3)] = v.t[1];
        m[(2, 3)] = v.t[2];
        m[(3, 3)] = R::one();
        m
    }
}

impl<'a, R> From<&'a Affine<R>> for [[R; 4]; 4]
where
    R: Float + NumAssign,
{
    fn from(value: &'a Affine<R>) -> Self {
        Into::<Mat4<R>>::into(value).0
    }
}

#[cfg(test)]
mod tests {
    use approx::ulps_eq;
    use cgmath::Matrix;
    use proptest::{prop_assert, proptest};

    use super::*;
    use crate::{
        quat::Quat,
        test_helpers::proptest::{affine, bounded_f32, bounded_nonzero_f32},
        vec::Vec4,
    };

    #[test]
    fn from_affine_for_mat_comparison() {
        let glamour_lhs = Affine::builder()
            .with_scale(1.5_f32)
            .with_translation(Vec4::new_vector(1.0_f32, 2.0, 3.0))
            .with_orientation(Quat::new(0.5_f32, 0.0, 1.0, 0.0))
            .build();

        let glamour_result: Mat4<f32> = glamour_lhs.into();
        let cgmath_lhs = cgmath::Decomposed {
            disp: cgmath::Vector3::new(glamour_lhs.t.x, glamour_lhs.t.y, glamour_lhs.t.z),
            rot: Into::<cgmath::Quaternion<f32>>::into(glamour_lhs.o.0),
            scale: glamour_lhs.s,
        };
        let cgmath_result: cgmath::Matrix4<f32> = cgmath_lhs.into();

        let nalgebra_lhs = nalgebra::Similarity3::from_parts(
            nalgebra::Translation3::new(glamour_lhs.t.x, glamour_lhs.t.y, glamour_lhs.t.z),
            nalgebra::Unit::from_quaternion(nalgebra::Quaternion::new(
                glamour_lhs.o.w,
                glamour_lhs.o.i,
                glamour_lhs.o.j,
                glamour_lhs.o.k,
            )),
            glamour_lhs.s,
        );
        let nalgebra_result = nalgebra_lhs.to_homogeneous();

        assert!(
            ulps_eq!(glamour_result, cgmath_result) && ulps_eq!(glamour_result, nalgebra_result),
            "\nglamour =     {glamour_result:?}\ncgmath   = {:?}\nnalgebra =         {:?}",
            cgmath_result.transpose(),
            nalgebra_result.transpose(),
        );
    }

    proptest! {
        #[test]
        #[ignore = "Nalgebra likely uses a different conversion algorithm which causes large rounding errors"]
        fn from_affine_for_mat_is_equal_to_nalgebra(glamour_lhs in affine(bounded_f32(-32, 32), bounded_nonzero_f32(-32, 32))) {
            let glamour_result: Mat4<f32> = glamour_lhs.into();
            let nalgebra_lhs = nalgebra::Similarity3::from_parts(
                nalgebra::Translation3::new(glamour_lhs.t.x, glamour_lhs.t.y, glamour_lhs.t.z),
                nalgebra::Unit::from_quaternion(nalgebra::Quaternion::new(glamour_lhs.o.w, glamour_lhs.o.i, glamour_lhs.o.j, glamour_lhs.o.k)),
                glamour_lhs.s,
            );
            let nalgebra_result = nalgebra_lhs.to_homogeneous();

            prop_assert!(
                ulps_eq!(glamour_result, nalgebra_result),
                "\nglamour  =    {glamour_result:?}\nnalgebra =         {:?}",
                nalgebra_result.transpose(),
            );
        }

        #[test]
        fn from_affine_for_mat_is_equal_to_cgmath(glamour_lhs in affine(bounded_f32(-32, 32), bounded_nonzero_f32(-32, 32))) {
            let glamour_result: Mat4<f32> = glamour_lhs.into();
            let cgmath_lhs = cgmath::Decomposed {
                disp: cgmath::Vector3::new(glamour_lhs.t.x, glamour_lhs.t.y, glamour_lhs.t.z),
                rot: Into::<cgmath::Quaternion<f32>>::into(glamour_lhs.o.0),
                scale: glamour_lhs.s,
            };
            let cgmath_result: cgmath::Matrix4<f32> = cgmath_lhs.into();

            prop_assert!(
                ulps_eq!(glamour_result, cgmath_result),
                "\nglamour =   {glamour_result:?}\ncgmath = {:?}",
                cgmath_result.transpose(),
            );
        }
    }
}
