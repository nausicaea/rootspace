use std::ops::{Add, Div, Mul, Neg, Sub};

use num_traits::{Float, Inv};
use crate::glamour::ops::inv_elem::InvElem;
use crate::glamour::ops::mul_elem::MulElem;

use super::super::Mat4;

macro_rules! impl_binops {
    ($($Op:ident::$op:ident => $Deleg:ident::$deleg:ident);+ $(;)*) => {
        $(
        impl<'a, 'b, R> $Op<&'b Mat4<R>> for &'a Mat4<R>
        where
            Mat4<R>: $crate::glamour::num::Zero,
            R: Copy + $Deleg<Output = R>,
        {
            type Output = Mat4<R>;

            fn $op(self, rhs: &'b Mat4<R>) -> Self::Output {
                use $crate::glamour::num::Zero;

                let mut mat = Mat4::<R>::zero();
                for i in 0..4 {
                    for j in 0..4 {
                        mat[(i, j)] = self[(i, j)].$deleg(rhs[(i, j)]);
                    }
                }
                mat
            }
        }

        $crate::forward_ref_binop!(impl<R: Float> $Op, $op for Mat4<R>, Mat4<R>, Mat4<R>);
        )+
    };
}

impl_binops! {
    Add::add => Add::add;
    Sub::sub => Sub::sub;
    MulElem::mul_elem => Mul::mul;
}

macro_rules! impl_unops {
    ($($Op:ident::$op:ident => $Deleg:ident::$deleg:ident);+ $(;)*) => {
        $(
        impl<R> $Op for Mat4<R>
        where
            Self: $crate::glamour::num::Zero,
            R: Copy + $Deleg<Output = R>,
        {
            type Output = Self;

            fn $op(self) -> Self::Output {
                (&self).$op()
            }
        }

        impl<'a, R> $Op for &'a Mat4<R>
        where
            Mat4<R>: $crate::glamour::num::Zero,
            R: Copy + $Deleg<Output = R>,
        {
            type Output = Mat4<R>;

            fn $op(self) -> Self::Output {
                use $crate::glamour::num::Zero;

                let mut mat = Mat4::<R>::zero();
                for i in 0..4 {
                    for j in 0..4 {
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
    Neg::neg => Neg::neg;
    InvElem::inv_elem => Inv::inv;
}

macro_rules! impl_scalar_binops {
    ($($Op:ident::$op:ident, [$($tgt:ident),+ $(,)*]);+ $(;)*) => {
        $(
        impl<'a, 'b, R> $Op<&'b R> for &'a Mat4<R>
            where
                Mat4<R>: $crate::glamour::num::Zero,
                R: Copy + $Op<R, Output = R>,
        {
            type Output = Mat4<R>;

            fn $op(self, rhs: &'b R) -> Self::Output {
                use $crate::glamour::num::Zero;

                let mut mat = Mat4::<R>::zero();
                for i in 0..4 {
                    for j in 0..4 {
                        mat[(i, j)] = $Op::$op(self[(i, j)], *rhs);
                    }
                }
                mat
            }
        }

        $crate::forward_ref_binop!(impl<R: Float> $Op, $op for Mat4<R>, R, Mat4<R>);

        $(
        impl<'a, 'b> $Op<&'b Mat4<$tgt>> for &'a $tgt {
            type Output = Mat4<$tgt>;

            fn $op(self, rhs: &'b Mat4<$tgt>) -> Self::Output {
                use $crate::glamour::num::Zero;

                let mut mat = Mat4::<$tgt>::zero();
                for i in 0..4 {
                    for j in 0..4 {
                        mat[(i, j)] = self.$op(rhs[(i, j)]);
                    }
                }
                mat
            }
        }

        $crate::forward_ref_binop!(impl $Op, $op for $tgt, Mat4<$tgt>, Mat4<$tgt>);
        )*

        )+
    }
}

impl_scalar_binops!(
    Add::add, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Sub::sub, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Mul::mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Div::div, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat4_supports_elementwise_neg() {
        let a: Mat4<f32> = Mat4::from([1.0; 16]);
        assert_eq!(-a, Mat4::<f32>::from([-1.0; 16]));
    }

    #[test]
    fn mat4_supports_scalar_addition() {
        let a: Mat4<f32> = Mat4::from([1.0; 16]);
        let b: f32 = 2.0;
        assert_eq!(&a + &b, Mat4::<f32>::from([3.0; 16]));
        assert_eq!(&b + &a, Mat4::<f32>::from([3.0; 16]));
    }

    #[test]
    fn mat4_supports_scalar_subtraction() {
        let a: Mat4<f32> = Mat4::from([1.0; 16]);
        let b: f32 = 2.0;
        assert_eq!(&a - &b, Mat4::<f32>::from([-1.0; 16]));
        assert_eq!(&b - &a, Mat4::<f32>::from([1.0; 16]));
    }

    #[test]
    fn mat4_supports_scalar_multiplication() {
        let a: Mat4<f32> = Mat4::from([2.0; 16]);
        let b: f32 = 2.0;
        assert_eq!(&a * &b, Mat4::<f32>::from([4.0; 16]));
        assert_eq!(&b * &a, Mat4::<f32>::from([4.0; 16]));
    }

    #[test]
    fn mat4_supports_scalar_division() {
        let a: Mat4<f32> = Mat4::from([6.0; 16]);
        let b: f32 = 2.0;
        assert_eq!(&a / &b, Mat4::<f32>::from([3.0; 16]));
        assert_eq!(&b / &a, Mat4::<f32>::from([2.0 / 6.0; 16]));
    }

    #[test]
    fn mat4_supports_matrix_addition() {
        let a: Mat4<f32> = Mat4::from([3.0; 16]);
        let b: Mat4<f32> = Mat4::from([2.0; 16]);
        assert_eq!(&a + &b, Mat4::<f32>::from([5.0; 16]));
        assert_eq!(&b + &a, Mat4::<f32>::from([5.0; 16]));
    }

    #[test]
    fn mat4_supports_matrix_subtraction() {
        let a: Mat4<f32> = Mat4::from([3.0; 16]);
        let b: Mat4<f32> = Mat4::from([2.0; 16]);
        assert_eq!(&a - &b, Mat4::<f32>::from([1.0; 16]));
        assert_eq!(&b - &a, Mat4::<f32>::from([-1.0; 16]));
    }
}
