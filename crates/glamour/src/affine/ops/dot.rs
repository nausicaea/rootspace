use forward_ref::forward_ref_binop;
use numenor::ConstZero;

use crate::{affine::Affine, mat::Mat4, num::CustomFloat, ops::dot::Dot};

impl<'b, R> Dot<&'b Affine<R>> for &Affine<R>
where
    R: CustomFloat + ConstZero,
{
    type Output = Mat4<R>;

    fn dot(self, rhs: &'b Affine<R>) -> Self::Output {
        Into::<Mat4<R>>::into(*self).dot(Into::<Mat4<R>>::into(*rhs))
    }
}

forward_ref_binop!(impl<R: CustomFloat> Dot, dot for Affine<R>, Affine<R>, Mat4<R>);
