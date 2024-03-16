use std::{iter::Product, ops::Mul};

use super::super::Mat4;
use crate::{
    forward_ref_binop,
    glamour::{iter_float::IterFloat, ops::dot::Dot, vec::Vec4},
};

impl<'a, 'b, R> Dot<&'b Mat4<R>> for &'a Mat4<R>
where
    R: IterFloat,
{
    type Output = Mat4<R>;

    fn dot(self, rhs: &'b Mat4<R>) -> Self::Output {
        let c = crate::abop!(
            dot,
            self,
            rhs,
            [
                ((0, 1, 2, 3), (0, 4, 8, 12)),
                ((0, 1, 2, 3), (1, 5, 9, 13)),
                ((0, 1, 2, 3), (2, 6, 10, 14)),
                ((0, 1, 2, 3), (3, 7, 11, 15)),
                ((4, 5, 6, 7), (0, 4, 8, 12)),
                ((4, 5, 6, 7), (1, 5, 9, 13)),
                ((4, 5, 6, 7), (2, 6, 10, 14)),
                ((4, 5, 6, 7), (3, 7, 11, 15)),
                ((8, 9, 10, 11), (0, 4, 8, 12)),
                ((8, 9, 10, 11), (1, 5, 9, 13)),
                ((8, 9, 10, 11), (2, 6, 10, 14)),
                ((8, 9, 10, 11), (3, 7, 11, 15)),
                ((12, 13, 14, 15), (0, 4, 8, 12)),
                ((12, 13, 14, 15), (1, 5, 9, 13)),
                ((12, 13, 14, 15), (2, 6, 10, 14)),
                ((12, 13, 14, 15), (3, 7, 11, 15)),
            ]
        );
        c.into()
    }
}

forward_ref_binop!(impl<R: IterFloat> Dot, dot for Mat4<R>, Mat4<R>, Mat4<R>);

impl<'a, 'b, R> Mul<&'b Mat4<R>> for &'a Mat4<R>
where
    &'a Mat4<R>: Dot<&'b Mat4<R>, Output = Mat4<R>>,
{
    type Output = Mat4<R>;

    fn mul(self, rhs: &'b Mat4<R>) -> Self::Output {
        self.dot(rhs)
    }
}

forward_ref_binop!(impl<R: IterFloat> Mul, mul for Mat4<R>, Mat4<R>, Mat4<R>);

impl<'a, 'b, R> Dot<&'b Vec4<R>> for &'a Mat4<R>
where
    R: IterFloat,
{
    type Output = Vec4<R>;

    fn dot(self, rhs: &'b Vec4<R>) -> Self::Output {
        let c = crate::abop!(
            dot,
            self,
            rhs,
            [
                ((0, 1, 2, 3), (0, 1, 2, 3)),
                ((4, 5, 6, 7), (0, 1, 2, 3)),
                ((8, 9, 10, 11), (0, 1, 2, 3)),
                ((12, 13, 14, 15), (0, 1, 2, 3)),
            ]
        );
        c.into()
    }
}

forward_ref_binop!(impl<R: IterFloat> Dot, dot for Mat4<R>, Vec4<R>, Vec4<R>);

impl<'a, 'b, R> Mul<&'b Vec4<R>> for &'a Mat4<R>
where
    &'a Mat4<R>: Dot<&'b Vec4<R>, Output = Vec4<R>>,
{
    type Output = Vec4<R>;

    fn mul(self, rhs: &'b Vec4<R>) -> Self::Output {
        self.dot(rhs)
    }
}

forward_ref_binop!(impl<R: IterFloat> Mul, mul for Mat4<R>, Vec4<R>, Vec4<R>);

impl<'a, R: IterFloat> Product<&'a Mat4<R>> for Mat4<R> {
    fn product<I: Iterator<Item = &'a Mat4<R>>>(iter: I) -> Self {
        iter.fold(Mat4::identity(), |state, item| state * item)
    }
}

impl<R: IterFloat> Product<Mat4<R>> for Mat4<R> {
    fn product<I: Iterator<Item = Mat4<R>>>(iter: I) -> Self {
        iter.fold(Mat4::identity(), |state, item| state * item)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use proptest::{prop_assert, proptest};

    use super::*;
    use crate::glamour::{
        num::One,
        test_helpers::{bounded_f32, bounded_nonzero_f32, mat4},
    };

    #[test]
    fn mat4_supports_dot_product_with_mat4() {
        let a: Mat4<f32> = Mat4::from([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        let b: Mat4<f32> = Mat4::from([
            2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
        ]);
        let c: Mat4<f32> = Mat4::from([
            100., 110., 120., 130., 228., 254., 280., 306., 356., 398., 440., 482., 484., 542., 600., 658.,
        ]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat4_supports_dot_product_with_vec4() {
        let a: Mat4<f32> = Mat4::from([
            2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
        ]);
        let b: Vec4<f32> = Vec4::from([1.0, 2.0, 3.0, 3.0]);
        let c: Vec4<f32> = Vec4::from([35.0, 71.0, 107.0, 143.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat4_x_vec4_works_by_postmultiplication() {
        let m: Mat4<f32> = Mat4::identity();
        let v: Vec4<f32> = Vec4::one();

        assert_eq!(m * v, Vec4::one());
    }

    proptest! {
        #[test]
        fn bounded_f32_mat_mul_does_not_cause_nans(lhs in mat4(bounded_f32(-62, 63)), rhs in mat4(bounded_f32(-62, 63))) {
            prop_assert!(!(lhs * rhs).is_nan());
        }

        #[test]
        fn bounded_nonzero_f32_mat_mul_does_not_cause_nans(lhs in mat4(bounded_nonzero_f32(-62, 63)), rhs in mat4(bounded_nonzero_f32(-62, 63))) {
            prop_assert!(!(lhs * rhs).is_nan());
        }

        /// Nalgebra's memory layout is column-major, while glamour is row-major. Therefore, transposition is necessary
        #[test]
        fn mat4_mul_is_equal_to_nalgebra(glamour_lhs in mat4(bounded_f32(-62, 63)), glamour_rhs in mat4(bounded_f32(-62, 63))) {
            let nalgebra_lhs: nalgebra::Matrix4<f32> = glamour_lhs.into();
            let nalgebra_rhs: nalgebra::Matrix4<f32> = glamour_rhs.into();
            let nalgebra_result: Mat4<f32> = (nalgebra_lhs * nalgebra_rhs).into();
            assert_ulps_eq!(glamour_lhs * glamour_rhs, nalgebra_result);
        }

        /// Cgmath's memory layout is column-major, while glamour is row-major. Therefore, transposition is necessary
        #[test]
        fn mat4_mul_is_equal_to_cgmath(glamour_lhs in mat4(bounded_f32(-62, 63)), glamour_rhs in mat4(bounded_f32(-62, 63))) {
            let cgmath_lhs: nalgebra::Matrix4<f32> = glamour_lhs.into();
            let cgmath_rhs: nalgebra::Matrix4<f32> = glamour_rhs.into();
            let cgmath_result: Mat4<f32> = (cgmath_lhs * cgmath_rhs).into();
            assert_ulps_eq!(glamour_lhs * glamour_rhs, cgmath_result);
        }
    }
}
