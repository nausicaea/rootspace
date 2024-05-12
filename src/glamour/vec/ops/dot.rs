use std::ops::Mul;

use num_traits::Float;

use super::super::Vec4;
use crate::{abop, forward_ref_binop, glamour::ops::dot::Dot};

impl<'a, 'b, R> Dot<&'b Vec4<R>> for &'a Vec4<R>
where
    R: Float,
{
    type Output = R;

    fn dot(self, rhs: &'b Vec4<R>) -> Self::Output {
        abop!(dot, self, rhs, [((0, 1, 2, 3), (0, 1, 2, 3))])[0]
    }
}

forward_ref_binop!(impl<R: Float> Dot, dot for Vec4<R>, Vec4<R>, R);

impl<'a, 'b, R> Mul<&'b Vec4<R>> for &'a Vec4<R>
where
    &'a Vec4<R>: Dot<&'b Vec4<R>, Output = R>,
{
    type Output = R;

    fn mul(self, rhs: &'b Vec4<R>) -> Self::Output {
        self.dot(rhs)
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Vec4<R>, Vec4<R>, R);
