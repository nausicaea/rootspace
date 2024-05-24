use crate::{
    num::{ConstOne, ConstZero, One, Zero},
    vec::Vec4,
};

impl<R> Zero for Vec4<R>
where
    R: num_traits::Zero,
{
    fn zero() -> Self {
        Vec4 {
            x: R::zero(),
            y: R::zero(),
            z: R::zero(),
            w: R::zero(),
        }
    }
}

impl<R> ConstZero for Vec4<R>
where
    R: num_traits::ConstZero,
{
    const ZERO: Self = Vec4 {
        x: R::ZERO,
        y: R::ZERO,
        z: R::ZERO,
        w: R::ZERO,
    };
}

impl<R> One for Vec4<R>
where
    R: num_traits::One,
{
    fn one() -> Self {
        Vec4 {
            x: R::one(),
            y: R::one(),
            z: R::one(),
            w: R::one(),
        }
    }
}

impl<R> ConstOne for Vec4<R>
where
    R: num_traits::ConstOne,
{
    const ONE: Vec4<R> = Vec4 {
        x: R::ONE,
        y: R::ONE,
        z: R::ONE,
        w: R::ONE,
    };
}

