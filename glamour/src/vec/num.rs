use crate::{One, Zero};

use super::super::Vec4;

impl<R> Zero for Vec4<R>
where
    R: Copy + num_traits::Zero,
{
    fn zero() -> Self {
        Vec4([R::zero(); 4])
    }
}

impl<R> One for Vec4<R>
where
    R: Copy + num_traits::One,
{
    fn one() -> Self {
        Vec4([R::one(); 4])
    }
}
