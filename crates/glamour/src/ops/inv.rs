use num_traits::{Float, Inv};

use crate::quat::Quat;

impl<R> Inv for Quat<R>
where
    R: Float + Inv<Output = R>,
{
    type Output = Self;

    fn inv(self) -> Self::Output {
        self.c() / self.abssq()
    }
}
