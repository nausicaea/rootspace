use num_traits::Num;
use crate::abop;

pub trait Dot<Rhs = Self> {
    type Output;

    fn dot(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_dot_product {
    ($tgt:ty, $tt:tt) => {
        impl<R> Dot for $tgt
        where
            R: Num + Copy + std::iter::Sum,
        {
            type Output = R;

            fn dot(self, rhs: Self) -> Self::Output {
                (&self).dot(&rhs)
            }
        }

        impl<'a, R> Dot for &'a $tgt 
            where
                R: Num + Copy + std::iter::Sum,
        {
            type Output = R;

            fn dot(self, rhs: Self) -> Self::Output {
                let c = abop!(mul, self, rhs, $tt);
                c.into_iter().sum()
            }
        }
    };
}

impl_dot_product!([R; 1], [(0, 0)]);
impl_dot_product!([R; 2], [(0, 0), (1, 1)]);
impl_dot_product!([R; 3], [(0, 0), (1, 1), (2, 2)]);
impl_dot_product!([R; 4], [(0, 0), (1, 1), (2, 2), (3, 3)]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_impls_dot() {
        let a = [1.0f32, 2.0f32];
        let b = [3.0f32, 4.0f32];
        assert_eq!(Dot::dot(a, b), 11.0f32);
    }
}
