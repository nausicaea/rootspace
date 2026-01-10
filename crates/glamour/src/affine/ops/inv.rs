use num_traits::Inv;

use crate::{affine::Affine, num::CustomFloat};

impl<R> Inv for Affine<R>
where
    R: CustomFloat + Inv<Output = R>,
{
    type Output = Self;

    fn inv(self) -> Self::Output {
        Self {
            t: -self.t,
            o: self.o.c().into(),
            s: self.s.inv(),
        }
    }
}
