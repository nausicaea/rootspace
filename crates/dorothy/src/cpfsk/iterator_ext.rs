use crate::cpfsk::integrate::{Integrate, integrate};
use crate::cpfsk::interpolate::Interpolate;

pub trait IteratorExt: Iterator {
    /// Smoothly interpolate values to a timescale `factor` times larger than the source. The first element of the interpolation is always `0.0`.
    ///
    /// # Examples
    /// ```
    /// use dorothy::IteratorExt;
    ///
    /// assert_eq!(
    ///     [0.0, 1.0].into_iter().interpolate(2).collect::<Vec<_>>(),
    ///     &[0.0, 0.0, 0.5, 1.0],
    /// );
    /// ```
    fn interpolate(self, factor: usize) -> Interpolate<Self>
    where
        Self: Sized + Iterator<Item = f64>,
    {
        Interpolate::new(factor, self)
    }

    /// Apply a cumulative summation over the source
    ///
    /// # Examples
    /// ```
    /// use dorothy::IteratorExt;
    ///
    /// assert_eq!(
    ///     [0.0, 1.0, 2.0].into_iter().integrate().collect::<Vec<_>>(),
    ///     &[0.0, 1.0, 3.0],
    /// );
    /// ```
    fn integrate(self) -> Integrate<Self>
    where
        Self: Sized + Iterator<Item = f64>,
    {
        integrate(self)
    }
}

impl<I: Iterator> IteratorExt for I {}
