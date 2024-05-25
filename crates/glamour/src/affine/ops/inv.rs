use num_traits::{Float, Inv};

use crate::affine::Affine;

impl<R> Inv for Affine<R>
where
    R: Float + Inv<Output = R>,
{
    type Output = Self;

    fn inv(self) -> Self::Output {
        Affine {
            t: -self.t,
            o: self.o.c().into(),
            s: self.s.inv(),
        }
    }
}
