use num_traits::Float;
use numenor::{ConstOne, ConstZero};

use super::mat::Mat4;

pub trait ToMatrix<N> {
    fn to_matrix(&self) -> Mat4<N>;
}

pub trait CustomFloat: Float + ConstZero + ConstOne {}

impl<T: Float + ConstZero + ConstOne> CustomFloat for T {}
