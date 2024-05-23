use forward_ref::forward_ref_binop;
use num_traits::Float;
use std::ops::Mul;

use crate::{affine::Affine, ops::dot::Dot};

impl<'a, 'b, R> Dot<&'b Affine<R>> for &'a Affine<R>
where
    R: Float,
{
    type Output = Affine<R>;

    fn dot(self, rhs: &'b Affine<R>) -> Self::Output {
        Affine {
            t: self.t + rhs.t,
            o: self.o * rhs.o,
            s: self.s * rhs.s,
        }
    }
}

forward_ref_binop!(impl<R: Float> Dot, dot for Affine<R>, Affine<R>, Affine<R>);

impl<'a, 'b, R> Mul<&'b Affine<R>> for &'a Affine<R>
where
    &'a Affine<R>: Dot<&'b Affine<R>, Output = Affine<R>>,
{
    type Output = Affine<R>;

    fn mul(self, rhs: &'b Affine<R>) -> Self::Output {
        self.dot(rhs)
    }
}

forward_ref_binop!(impl<R: Float> Mul, mul for Affine<R>, Affine<R>, Affine<R>);
