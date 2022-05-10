use std::ops::{Add, Sub, Neg, Mul, Div};
use num_traits::{Float, Inv};
use super::super::Mat;
use crate::ops::mul_elem::MulElem;
use crate::ops::inv_elem::InvElem;
use forward_ref::forward_ref_binop;

macro_rules! impl_binops {
    ($($Op:ident, $op:ident, $deleg:ident);+ $(;)*) => {
        $(
        impl<'a, 'b, R, const I: usize, const J: usize> $Op<&'b Mat<R, I, J>> for &'a Mat<R, I, J>
        where
            R: Float,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: &'b Mat<R, I, J>) -> Self::Output {
                let mut mat = Mat::<R, I, J>::zero();
                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = self[(i, j)].$deleg(rhs[(i, j)]);
                    }
                }
                mat
            }
        }

        forward_ref_binop!(impl<R: Float, const I: usize, const J: usize> $Op, $op for Mat<R, I, J>, Mat<R, I, J>, Mat<R, I, J>);
        )+
    };
}

impl_binops! {
    Add, add, add;
    Sub, sub, sub;
    MulElem, mul_elem, mul;
}

macro_rules! impl_unops {
    ($($Op:ident, $op:ident, $deleg:ident);+ $(;)*) => {
        $(
        impl<R, const I: usize, const J: usize> $Op for Mat<R, I, J>
        where
            R: Float + Inv<Output = R>,
        {
            type Output = Self;

            fn $op(self) -> Self::Output {
                (&self).$op()
            }
        }

        impl<'a, R, const I: usize, const J: usize> $Op for &'a Mat<R, I, J>
        where
            R: Float + Inv<Output = R>,
        {
            type Output = Mat<R, I, J>;

            fn $op(self) -> Self::Output {
                let mut mat = Mat::<R, I, J>::zero();
                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = self[(i, j)].$deleg();
                    }
                }
                mat
            }
        }
        )+
    };
}

impl_unops! {
    Neg, neg, neg;
    InvElem, inv_elem, inv;
}

macro_rules! impl_scalar_binops {
    ($($Op:ident, $op:ident, [$($tgt:ident),+ $(,)*]);+ $(;)*) => {
        $(
        impl<'a, 'b, R, const I: usize, const J: usize> $Op<&'b R> for &'a Mat<R, I, J>
            where
                R: Float,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: &'b R) -> Self::Output {
                let mut mat = Mat::<R, I, J>::zero();
                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = self[(i, j)].$op(*rhs);
                    }
                }
                mat
            }
        }

        forward_ref_binop!(impl<R: Float, const I: usize, const J: usize> $Op, $op for Mat<R, I, J>, R, Mat<R, I, J>);

        $(
        impl<'a, 'b, const I: usize, const J: usize> $Op<&'b Mat<$tgt, I, J>> for &'a $tgt {
            type Output = Mat<$tgt, I, J>;

            fn $op(self, rhs: &'b Mat<$tgt, I, J>) -> Self::Output {
                let mut mat = Mat::<$tgt, I, J>::zero();
                for i in 0..I {
                    for j in 0..J {
                        mat[(i, j)] = self.$op(rhs[(i, j)]);
                    }
                }
                mat
            }
        }

        forward_ref_binop!(impl<const I: usize, const J: usize> $Op, $op for $tgt, Mat<$tgt, I, J>, Mat<$tgt, I, J>);
        )*

        )+
    }
}

impl_scalar_binops!(
    Add, add, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Sub, sub, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Mul, mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Div, div, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat_supports_scalar_addition() {
        let a: Mat<f32, 1, 1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a + &b, Mat::<f32, 1, 1>::from([3.0]));
        assert_eq!(&b + &a, Mat::<f32, 1, 1>::from([3.0]));
    }

    #[test]
    fn mat_supports_scalar_subtraction() {
        let a: Mat<f32, 1, 1> = Mat::from([1.0]);
        let b: f32 = 2.0;
        assert_eq!(&a - &b, Mat::<f32, 1, 1>::from([-1.0]));
        assert_eq!(&b - &a, Mat::<f32, 1, 1>::from([1.0]));
    }

    #[test]
    fn mat_supports_scalar_multiplication() {
        let a: Mat<f32, 1, 1> = Mat::from([2.0]);
        let b: f32 = 2.0;
        assert_eq!(&a * &b, Mat::<f32, 1, 1>::from([4.0]));
        assert_eq!(&b * &a, Mat::<f32, 1, 1>::from([4.0]));
    }

    #[test]
    fn mat_supports_scalar_division() {
        let a: Mat<f32, 1, 1> = Mat::from([6.0]);
        let b: f32 = 2.0;
        assert_eq!(&a / &b, Mat::<f32, 1, 1>::from([3.0]));
        assert_eq!(&b / &a, Mat::<f32, 1, 1>::from([2.0 / 6.0]));
    }

    #[test]
    fn mat_supports_matrix_addition() {
        let a: Mat<f32, 1, 1> = Mat::from([3.0]);
        let b: Mat<f32, 1, 1> = Mat::from([2.0]);
        assert_eq!(&a + &b, Mat::<f32, 1, 1>::from([5.0]));
        assert_eq!(&b + &a, Mat::<f32, 1, 1>::from([5.0]));
    }

    #[test]
    fn mat_supports_matrix_subtraction() {
        let a: Mat<f32, 1, 1> = Mat::from([3.0]);
        let b: Mat<f32, 1, 1> = Mat::from([2.0]);
        assert_eq!(&a - &b, Mat::<f32, 1, 1>::from([1.0]));
        assert_eq!(&b - &a, Mat::<f32, 1, 1>::from([-1.0]));
    }
}
