use num_traits::Float;
use std::iter::Sum;

pub trait FloatAndSum: Float + Sum<Self> + for<'r> Sum<&'r Self> {}

impl<T> FloatAndSum for T
where
    T: Float + Sum<Self> + for<'r> Sum<&'r Self>,
{
}

