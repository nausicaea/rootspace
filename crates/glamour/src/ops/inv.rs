use crate::num::CustomFloat;
use num_traits::Inv;

use crate::quat::Quat;

impl<R> Inv for Quat<R>
where
    R: CustomFloat + Inv<Output = R>,
{
    type Output = Self;

    fn inv(self) -> Self::Output {
        self.c() / self.abssq()
    }
}
