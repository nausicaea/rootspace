use num_traits::Float;

use crate::{ops::cross::Cross, vec::Vec4};

impl<'b, R> Cross<&'b Vec4<R>> for &Vec4<R>
where
    R: Copy + num_traits::Zero + std::ops::Mul<R, Output = R> + std::ops::Sub<R, Output = R>,
{
    type Output = Vec4<R>;

    fn cross(self, rhs: &'b Vec4<R>) -> Self::Output {
        Vec4 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
            w: R::zero(),
        }
    }
}

forward_ref::forward_ref_binop!(impl<R: Float> Cross, cross for Vec4<R>, Vec4<R>, Vec4<R>);

#[cfg(test)]
mod tests {
    use approx::ulps_eq;
    use proptest::{prop_assert, proptest};

    use crate::{
        ops::cross::Cross,
        test_helpers::proptest::{bounded_f32, vec4},
    };

    proptest! {
        #[test]
        fn vec4_cross_is_equal_to_nalgebra(lhs in vec4(bounded_f32(-32, 32)), rhs in vec4(bounded_f32(-32, 32))) {
            let glamour_result = lhs.cross(rhs);
            let nalgebra_result = nalgebra::Vector3::new(lhs.x, lhs.y, lhs.z).cross(&nalgebra::Vector3::new(rhs.x, rhs.y, rhs.z));
            let nalgebra_result = nalgebra::Vector4::new(nalgebra_result.x, nalgebra_result.y, nalgebra_result.z, 0.0);

            prop_assert!(
                ulps_eq!(glamour_result, nalgebra_result),
                "\nglamour = {}\nnalgebra = {}",
                glamour_result,
                nalgebra_result,
            );
        }

        #[test]
        fn vec4_cross_is_equal_to_cgmath(lhs in vec4(bounded_f32(-32, 32)), rhs in vec4(bounded_f32(-32, 32))) {
            let glamour_result = lhs.cross(rhs);
            let cgmath_result = cgmath::Vector3::new(lhs.x, lhs.y, lhs.z).cross(cgmath::Vector3::new(rhs.x, rhs.y, rhs.z));
            let cgmath_result = cgmath::Vector4::new(cgmath_result.x, cgmath_result.y, cgmath_result.z, 0.0);

            prop_assert!(
                ulps_eq!(glamour_result, cgmath_result),
                "\nglamour = {:?}\ncgmath = {:?}",
                glamour_result,
                cgmath_result,
            );
        }
    }
}
