use std::ops::{Add, Sub, Neg, Mul, Div};
use num_traits::{Float, Inv};
use super::Mat;
use crate::mul_elem::MulElem;
use crate::inv_elem::InvElem;

macro_rules! impl_binops {
    ($($Op:ident, $op:ident, $deleg:ident);+ $(;)*) => {
        $(
        impl<R, const I: usize, const J: usize> $Op for Mat<R, I, J>
        where
            R: Float,
        {
            type Output = Self;

            fn $op(self, rhs: Self) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b, R, const I: usize, const J: usize> $Op<&'b Mat<R, I, J>> for Mat<R, I, J>
        where
            R: Float,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: &'b Mat<R, I, J>) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a, R, const I: usize, const J: usize> $Op<Mat<R, I, J>> for &'a Mat<R, I, J>
        where
            R: Float,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: Mat<R, I, J>) -> Self::Output {
                self.$op(&rhs)
            }
        }

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
        impl<R, const I: usize, const J: usize> $Op<R> for Mat<R, I, J>
            where
                R: Float,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: R) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b, R, const I: usize, const J: usize> $Op<&'b R> for Mat<R, I, J>
            where
                R: Float,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: &'b R) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a, R, const I: usize, const J: usize> $Op<R> for &'a Mat<R, I, J>
            where
                R: Float,
        {
            type Output = Mat<R, I, J>;

            fn $op(self, rhs: R) -> Self::Output {
                self.$op(&rhs)
            }
        }

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

        $(
        impl<const I: usize, const J: usize> $Op<Mat<$tgt, I, J>> for $tgt {
            type Output = Mat<$tgt, I, J>;

            fn $op(self, rhs: Mat<$tgt, I, J>) -> Self::Output {
                (&self).$op(&rhs)
            }
        }

        impl<'b, const I: usize, const J: usize> $Op<&'b Mat<$tgt, I, J>> for $tgt {
            type Output = Mat<$tgt, I, J>;

            fn $op(self, rhs: &'b Mat<$tgt, I, J>) -> Self::Output {
                (&self).$op(rhs)
            }
        }

        impl<'a, const I: usize, const J: usize> $Op<Mat<$tgt, I, J>> for &'a $tgt {
            type Output = Mat<$tgt, I, J>;

            fn $op(self, rhs: Mat<$tgt, I, J>) -> Self::Output {
                self.$op(&rhs)
            }
        }

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

