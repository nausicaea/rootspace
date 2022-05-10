use num_traits::Float;
use std::iter::{Sum, Product};

pub trait IterFloat: Float + Sum<Self> + for<'r> Sum<&'r Self> + Product<Self> + for<'r> Product<&'r Self> {}

impl<T> IterFloat for T
where
    T: Float + Sum<Self> + for<'r> Sum<&'r Self> + Product<Self> + for<'r> Product<&'r Self>,
{
}
