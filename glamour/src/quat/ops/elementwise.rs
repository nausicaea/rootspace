use std::ops::{Add, Div, Mul, Sub};

use num_traits::Float;

use super::super::Quat;

macro_rules! impl_scalar_binops {
    ($($Op:ident::$op:ident, [$($tgt:ident),+ $(,)*]);+ $(;)*) => {
        $(
        impl<'a, 'b, R> $Op<&'b R> for &'a Quat<R>
        where
            R: Copy + $Op<R, Output = R>,
        {
            type Output = Quat<R>;

            fn $op(self, rhs: &'b R) -> Self::Output {
                Quat {
                    w: $Op::$op(self.w, *rhs),
                    i: $Op::$op(self.i, *rhs),
                    j: $Op::$op(self.j, *rhs),
                    k: $Op::$op(self.k, *rhs),
                }
            }
        }

        forward_ref::forward_ref_binop!(impl<R: Float> $Op, $op for Quat<R>, R, Quat<R>);

        $(
        impl<'a, 'b> $Op<&'b Quat<$tgt>> for &'a $tgt {
            type Output = Quat<$tgt>;

            fn $op(self, rhs: &'b Quat<$tgt>) -> Self::Output {
                Quat {
                    w: $Op::$op(self, rhs.w),
                    i: $Op::$op(self, rhs.i),
                    j: $Op::$op(self, rhs.j),
                    k: $Op::$op(self, rhs.k),
                }
            }
        }

        forward_ref::forward_ref_binop!(impl $Op, $op for $tgt, Quat<$tgt>, Quat<$tgt>);
        )*

        )+
    }
}

impl_scalar_binops! {
    Add::add, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Sub::sub, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Mul::mul, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
    Div::div, [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quat_supports_scalar_multiplication() {
        let a: Quat<f32> = Quat::new(1.0, 1.0, 1.0, 1.0);
        let b: f32 = 2.0;
        assert_eq!(a * b, Quat::<f32>::new(2.0, 2.0, 2.0, 2.0));
        assert_eq!(b * a, Quat::<f32>::new(2.0, 2.0, 2.0, 2.0));
    }
}
