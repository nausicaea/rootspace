use forward_ref::forward_ref_binop;
use num_traits::Float;

use crate::{affine::Affine, mat::Mat4, ops::dot::Dot};

impl<'a, 'b, R> Dot<&'b Affine<R>> for &'a Affine<R>
where
    R: Float,
{
    type Output = Mat4<R>;

    fn dot(self, rhs: &'b Affine<R>) -> Self::Output {
        Into::<Mat4<R>>::into(*self).dot(Into::<Mat4<R>>::into(*rhs))
    }
}

forward_ref_binop!(impl<R: Float> Dot, dot for Affine<R>, Affine<R>, Mat4<R>);
