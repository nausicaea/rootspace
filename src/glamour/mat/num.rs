use crate::glamour::num::{One, Zero};

use super::Mat4;

impl<R> Zero for Mat4<R>
where
    R: Copy + num_traits::Zero,
{
    fn zero() -> Self {
        Mat4([[R::zero(); 4]; 4])
    }
}

impl<R> One for Mat4<R>
where
    R: Copy + num_traits::One,
{
    fn one() -> Self {
        Mat4([[R::one(); 4]; 4])
    }
}
