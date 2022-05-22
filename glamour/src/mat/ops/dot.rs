use crate::ops::dot::Dot;
use crate::abop;
use super::super::Mat;
use std::ops::Mul;
use forward_ref::forward_ref_binop;
use crate::iter_float::IterFloat;

macro_rules! impl_matmul {
    ($dim:literal, $tt:tt) => {
        impl_matmul!($dim, $dim, $dim, $dim, $tt);
    };
    ($nl:literal, $ml:literal, $nr:literal, $mr:literal, $tt:tt) => {
        impl<'a, 'b, R> Mul<&'b Mat<R, $nr, $mr>> for &'a Mat<R, $nl, $ml>
            where
                R: IterFloat,
        {
            type Output = Mat<R, $nl, $mr>;

            fn mul(self, rhs: &'b Mat<R, $nr, $mr>) -> Self::Output {
                self.dot(rhs)
            }
        }

        forward_ref_binop!(impl<R: IterFloat> Mul, mul for Mat<R, $nl, $ml>, Mat<R, $nr, $mr>, Mat<R, $nl, $mr>);

        impl<'a, 'b, R> Dot<&'b Mat<R, $nr, $mr>> for &'a Mat<R, $nl, $ml>
        where
            R: IterFloat,
        {
            type Output = Mat<R, $nl, $mr>;

            fn dot(self, rhs: &'b Mat<R, $nr, $mr>) -> Self::Output {
                let c = abop!(dot, self, rhs, $tt);
                c.into()
            }
        }

        forward_ref_binop!(impl<R: IterFloat> Dot, dot for Mat<R, $nl, $ml>, Mat<R, $nr, $mr>, Mat<R, $nl, $mr>);
    };
}

impl_matmul!(2, 1, 1, 2, [((0), (0)), ((0), (1)), ((1), (0)), ((1), (1))]);

impl_matmul!(
    2,
    [((0, 1), (0, 2)), ((0, 1), (1, 3)), ((2, 3), (0, 2)), ((2, 3), (1, 3)),]
);
impl_matmul!(1, 2, 2, 2, [((0, 1), (0, 2)), ((0, 1), (1, 3))]);
impl_matmul!(2, 2, 2, 1, [((0, 1), (0, 1)), ((2, 3), (0, 1))]);

impl_matmul!(
    3,
    [
        ((0, 1, 2), (0, 3, 6)),
        ((0, 1, 2), (1, 4, 7)),
        ((0, 1, 2), (2, 5, 8)),
        ((3, 4, 5), (0, 3, 6)),
        ((3, 4, 5), (1, 4, 7)),
        ((3, 4, 5), (2, 5, 8)),
        ((6, 7, 8), (0, 3, 6)),
        ((6, 7, 8), (1, 4, 7)),
        ((6, 7, 8), (2, 5, 8)),
    ]
);
impl_matmul!(
    1, 3, 3, 3,
    [
        ((0, 1, 2), (0, 3, 6)),
        ((0, 1, 2), (1, 4, 7)),
        ((0, 1, 2), (2, 5, 8)),
    ]
);
impl_matmul!(
    3, 3, 3, 1,
    [
        ((0, 1, 2), (0, 1, 2)),
        ((3, 4, 5), (0, 1, 2)),
        ((6, 7, 8), (0, 1, 2)),
    ]
);

impl_matmul!(
    4,
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
impl_matmul!(
    1, 4, 4, 4,
    [
        ((0, 1, 2, 3), (0, 4, 8, 12)),
        ((0, 1, 2, 3), (1, 5, 9, 13)),
        ((0, 1, 2, 3), (2, 6, 10, 14)),
        ((0, 1, 2, 3), (3, 7, 11, 15)),
    ]
);
impl_matmul!(
    4, 4, 4, 1,
    [
        ((0, 1, 2, 3), (0, 1, 2, 3)),
        ((4, 5, 6, 7), (0, 1, 2, 3)),
        ((8, 9, 10, 11), (0, 1, 2, 3)),
        ((12, 13, 14, 15), (0, 1, 2, 3)),
    ]
);

impl<'a, 'b, R> Mul<&'b Mat<R, 2, 1>> for &'a Mat<R, 1, 2>
where
    R: IterFloat
{
    type Output = R;

    fn mul(self, rhs: &'b Mat<R, 2, 1>) -> Self::Output {
        self.dot(rhs)
    }
}

forward_ref_binop!(impl<R: IterFloat> Mul, mul for Mat<R, 1, 2>, Mat<R, 2, 1>, R);

/// MARK
impl<'a, 'b, R> Dot<&'b Mat<R, 2, 1>> for &'a Mat<R, 1, 2>
where
    R: IterFloat
{
    type Output = R;

    fn dot(self, rhs: &'b Mat<R, 2, 1>) -> Self::Output {
        abop!(dot, self, rhs, [((0, 1), (0, 1))])[0]
    }
}

forward_ref_binop!(impl<R: IterFloat> Dot, dot for Mat<R, 1, 2>, Mat<R, 2, 1>, R);


#[cfg(test)]
mod tests {
    use super::*;
    use crate::mat::{Mat2, Mat3, Mat4};
    use crate::mat::vec2::Vec2;
    use crate::mat::vec3::Vec3;
    use crate::mat::vec4::Vec4;

    #[test]
    fn mat_supports_dot_product_2x1_1x2() {
        let a: Mat<f32, 2, 1> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, 1, 2> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), Mat::<f32, 2, 2>::from([6.0, 3.0, 4.0, 2.0]));
        assert_eq!(a.clone().dot(&b), Mat::<f32, 2, 2>::from([6.0, 3.0, 4.0, 2.0]));
        assert_eq!((&a).dot(b.clone()), Mat::<f32, 2, 2>::from([6.0, 3.0, 4.0, 2.0]));
        assert_eq!(a.clone().dot(b.clone()), Mat::<f32, 2, 2>::from([6.0, 3.0, 4.0, 2.0]));
    }

    #[test]
    fn mat_supports_dot_product_1x2_2x1() {
        let a: Mat<f32, 1, 2> = Mat::from([3.0, 2.0]);
        let b: Mat<f32, 2, 1> = Mat::from([2.0, 1.0]);
        assert_eq!((&a).dot(&b), 8.0f32);
    }

    #[test]
    fn mat_supports_dot_product_2x2_2x2() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 2, 2> = Mat::from([2.0, 3.0, 4.0, 5.0]);
        let c: Mat<f32, 2, 2> = Mat::from([10.0, 13.0, 22.0, 29.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x2_2x2() {
        let a: Mat<f32, 1, 2> = Mat::from([2.0, 3.0]);
        let b: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let c: Mat<f32, 1, 2> = Mat::from([11.0, 16.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_2x2_2x1() {
        let a: Mat<f32, 2, 2> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 2, 1> = Mat::from([2.0, 3.0]);
        let c: Mat<f32, 2, 1> = Mat::from([8.0, 18.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_3x3_3x3() {
        let a: Mat<f32, 3, 3> = Mat::from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let b: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let c: Mat<f32, 3, 3> = Mat::from([36., 42., 48., 81., 96., 111., 126., 150., 174.]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x3_3x3() {
        let a: Mat<f32, 1, 3> = Mat::from([1.0, 2.0, 3.0]);
        let b: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let c: Mat<f32, 1, 3> = Mat::from([36.0, 42.0, 48.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_3x3_3x1() {
        let a: Mat<f32, 3, 3> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let b: Mat<f32, 3, 1> = Mat::from([1.0, 2.0, 3.0]);
        let c: Mat<f32, 3, 1> = Mat::from([20.0, 38.0, 56.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_4x4_4x4() {
        let a: Mat<f32, 4, 4> = Mat::from([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        let b: Mat<f32, 4, 4> = Mat::from([
            2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
        ]);
        let c: Mat<f32, 4, 4> = Mat::from([
            100., 110., 120., 130., 228., 254., 280., 306., 356., 398., 440., 482., 484., 542., 600., 658.,
        ]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_1x4_4x4() {
        let a: Mat<f32, 1, 4> = Mat::from([1.0, 2.0, 3.0, 4.0]);
        let b: Mat<f32, 4, 4> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let c: Mat<f32, 1, 4> = Mat::from([100.0, 110.0, 120.0, 130.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat_supports_dot_product_4x4_4x1() {
        let a: Mat<f32, 4, 4> = Mat::from([2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0]);
        let b: Mat<f32, 4, 1> = Mat::from([1.0, 2.0, 3.0, 3.0]);
        let c: Mat<f32, 4, 1> = Mat::from([35.0, 71.0, 107.0, 143.0]);
        assert_eq!((&a).dot(&b), c);
    }

    #[test]
    fn mat2_x_vec2_works_as_premultiplication_of_the_matrix() {
        let m: Mat2<f32> = Mat2::identity();
        let v: Vec2<f32> = Vec2::one();

        assert_eq!(m * v, Vec2::one());
    }

    #[test]
    fn mat3_x_vec3_works_as_premultiplication_of_the_matrix() {
        let m: Mat3<f32> = Mat3::identity();
        let v: Vec3<f32> = Vec3::one();

        assert_eq!(m * v, Vec3::one());
    }

    #[test]
    fn mat4_x_vec4_works_as_premultiplication_of_the_matrix() {
        let m: Mat4<f32> = Mat4::identity();
        let v: Vec4<f32> = Vec4::one();

        assert_eq!(m * v, Vec4::one());
    }

}
