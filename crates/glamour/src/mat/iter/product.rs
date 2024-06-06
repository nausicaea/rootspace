use std::iter::Product;

use num_traits::Float;

use crate::mat::Mat4;

impl<'a, R: Float> Product<&'a Mat4<R>> for Mat4<R> {
    fn product<I: Iterator<Item = &'a Mat4<R>>>(iter: I) -> Self {
        iter.fold(Mat4::identity(), |state, item| state * item)
    }
}

impl<R: Float> Product<Mat4<R>> for Mat4<R> {
    fn product<I: Iterator<Item = Mat4<R>>>(iter: I) -> Self {
        iter.fold(Mat4::identity(), |state, item| state * item)
    }
}

