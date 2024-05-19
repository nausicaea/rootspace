use std::ops::{Add, Div, Mul, Neg, Sub};

use num_traits::{Float, Inv};

use crate::{
    ops::{inv_elem::InvElem, mul_elem::MulElem},
    vec::Vec4,
};

macro_rules! impl_binops {
    ($($Op:ident::$op:ident => $Deleg:ident::$deleg:ident);+ $(;)*) => {
        $(
        impl<'a, 'b, R> $Op<&'b Vec4<R>> for &'a Vec4<R>
        where
            Vec4<R>: $crate::num::Zero,
            R: Copy + $Deleg<Output = R>,
        {
            type Output = Vec4<R>;

            fn $op(self, rhs: &'b Vec4<R>) -> Self::Output {
                use $crate::num::Zero;

                let mut mat = Vec4::<R>::zero();
                for i in 0..4 {
                    mat[i] = self[i].$deleg(rhs[i]);
                }
                mat
            }
        }

        forward_ref::forward_ref_binop!(impl<R: Float> $Op, $op for Vec4<R>, Vec4<R>, Vec4<R>);
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
        impl<R> $Op for Vec4<R>
        where
            Self: $crate::num::Zero,
            R: Copy + $Deleg<Output = R>,
        {
            type Output = Self;

            fn $op(self) -> Self::Output {
                (&self).$op()
            }
        }

        impl<'a, R> $Op for &'a Vec4<R>
        where
            Vec4<R>: $crate::num::Zero,
            R: Copy + $Deleg<Output = R>,
        {
            type Output = Vec4<R>;

            fn $op(self) -> Self::Output {
                use $crate::num::Zero;

                let mut mat = Vec4::<R>::zero();
                for i in 0..4 {
                    mat[i] = self[i].$deleg();
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
        impl<'a, 'b, R> $Op<&'b R> for &'a Vec4<R>
            where
                Vec4<R>: $crate::num::Zero,
                R: Copy + $Op<Output = R>,
        {
            type Output = Vec4<R>;

            fn $op(self, rhs: &'b R) -> Self::Output {
                use $crate::num::Zero;

                let mut mat = Vec4::<R>::zero();
                for i in 0..4 {
                    mat[i] = self[i].$op(*rhs);
                }
                mat
            }
        }

        forward_ref::forward_ref_binop!(impl<R: Float> $Op, $op for Vec4<R>, R, Vec4<R>);

        $(
        impl<'a, 'b> $Op<&'b Vec4<$tgt>> for &'a $tgt {
            type Output = Vec4<$tgt>;

            fn $op(self, rhs: &'b Vec4<$tgt>) -> Self::Output {
                use $crate::num::Zero;

                let mut mat = Vec4::<$tgt>::zero();
                for i in 0..4 {
                    mat[i] = self.$op(rhs[i]);
                }
                mat
            }
        }

        forward_ref::forward_ref_binop!(impl $Op, $op for $tgt, Vec4<$tgt>, Vec4<$tgt>);
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
