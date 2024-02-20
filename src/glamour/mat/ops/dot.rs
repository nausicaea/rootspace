use crate::forward_ref_binop;
use crate::glamour::iter_float::IterFloat;
use crate::glamour::ops::dot::Dot;
use crate::glamour::vec::Vec4;
use std::ops::Mul;

use super::super::Mat4;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glamour::num::One;

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
}
