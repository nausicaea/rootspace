use std::iter::Product;

use num_traits::Float;

use crate::affine::Affine;

impl<R> Product for Affine<R> 
where
    R: Float,
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Affine::identity(), |state, elem| state * elem)
    }
}

impl<'a, R> Product<&'a Affine<R>> for Affine<R>
where
    R: Float,
{
    fn product<I: Iterator<Item = &'a Affine<R>>>(iter: I) -> Self {
        iter.fold(Affine::identity(), |state, elem| state * elem)
    }
}
