use forward_ref::forward_ref_binop;
use num_traits::Float;
use std::ops::Mul;

use crate::{affine::Affine, mat::Mat4, ops::dot::Dot};

impl<'a, 'b, R> Mul<&'b Affine<R>> for &'a Affine<R>
where
    &'a Affine<R>: Dot<&'b Affine<R>, Output = Mat4<R>>,
{
    type Output = Mat4<R>;

    fn mul(self, rhs: &'b Affine<R>) -> Self::Output {
        self.dot(rhs)
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Affine<R>, Affine<R>, Mat4<R>);
