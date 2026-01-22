use std::iter::Product;

use crate::{mat::Mat4, num::CustomFloat};

impl<'a, R: CustomFloat> Product<&'a Self> for Mat4<R> {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::identity(), |state, item| state * item)
    }
}

impl<R: CustomFloat> Product<Self> for Mat4<R> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::identity(), |state, item| state * item)
    }
}
