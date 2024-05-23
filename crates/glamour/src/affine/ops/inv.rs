use num_traits::{Float, Inv};

use crate::affine::Affine;

impl<R> Inv for Affine<R>
where
    R: Float,
{
    type Output = Self;

    fn inv(self) -> Self::Output {
        Affine {
            t: -self.t,
            o: self.o.inv().into(),
            s: R::one() / self.s,
        }
    }
}
