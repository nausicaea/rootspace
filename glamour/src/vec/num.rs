use crate::{
    num::{One, Zero},
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
