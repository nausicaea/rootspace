use num_traits::Float;

use super::Unit;
use crate::{mat::Mat4, ops::norm::Norm, quat::Quat, vec::Vec4};

impl<R> From<Quat<R>> for Unit<Quat<R>>
where
    Quat<R>: Norm<Output = R>,
    R: Float,
{
    fn from(v: Quat<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}

impl<R> From<Mat4<R>> for Unit<Mat4<R>>
where
    Mat4<R>: Norm<Output = R>,
    R: Float,
{
    fn from(v: Mat4<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}

impl<R> From<Vec4<R>> for Unit<Vec4<R>>
where
    Vec4<R>: Norm<Output = R>,
    R: Float,
{
    fn from(v: Vec4<R>) -> Self {
        let norm = v.norm();
        Unit(v / norm)
    }
}

impl<R> From<Unit<Self>> for Quat<R> {
    fn from(value: Unit<Self>) -> Self {
        value.0
    }
}

impl<R> From<Unit<Self>> for Mat4<R> {
    fn from(value: Unit<Self>) -> Self {
        value.0
    }
}

impl<R> From<Unit<Self>> for Vec4<R> {
    fn from(value: Unit<Self>) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use approx::ulps_eq;
    use cgmath::InnerSpace;
    use proptest::{prop_assert, proptest};

    use crate::{test_helpers::proptest::{bounded_nonzero_f32, vec4}, unit::Unit};

    proptest! {
        #[test]
        fn from_vec4_for_unit_is_equal_to_nalgebra(v in vec4(bounded_nonzero_f32(-32, 32))) {
            let glamour_result: Unit<_> = v.into();
            let nalgebra_result = nalgebra::UnitVector4::new_normalize(nalgebra::Vector4::new(v.x, v.y, v.z, v.w));

            prop_assert!(
                ulps_eq!(glamour_result.0, nalgebra_result.into_inner()),
                "\nglamour = {:?}\nnalgebra = {:?}",
                glamour_result,
                nalgebra_result,
            );
        }

        #[test]
        fn from_vec4_for_unit_is_equal_to_cgmath(v in vec4(bounded_nonzero_f32(-32, 32))) {
            let glamour_result: Unit<_> = v.into();
            let cgmath_result = cgmath::Vector4::new(v.x, v.y, v.z, v.w).normalize();

            prop_assert!(
                ulps_eq!(glamour_result.0, cgmath_result),
                "\nglamour = {:?}\ncgmath = {:?}",
                glamour_result,
                cgmath_result,
            );
        }
    }
}
