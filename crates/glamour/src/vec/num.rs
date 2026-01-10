use numenor::{ConstOne, ConstZero};

use crate::vec::Vec4;

impl<R> ConstZero for Vec4<R>
where
    R: ConstZero,
{
    const ZERO: Self = Self {
        x: R::ZERO,
        y: R::ZERO,
        z: R::ZERO,
        w: R::ZERO,
    };
}

impl<R> ConstOne for Vec4<R>
where
    R: ConstOne,
{
    const ONE: Self = Self {
        x: R::ONE,
        y: R::ONE,
        z: R::ONE,
        w: R::ONE,
    };
}
