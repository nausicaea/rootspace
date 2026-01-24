use crate::cpfsk::integrate::{Integrate, integrate};
use crate::cpfsk::interpolate::Interpolate;
use crate::cpfsk::windowed::Windowed;

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

    fn center(self) -> impl Iterator<Item = f64>
    where
        Self: Sized + Iterator<Item = f64>,
    {
        let signal = self.collect::<Vec<_>>();
        #[allow(clippy::cast_precision_loss)]
        let dc_offset = signal.iter().sum::<f64>() / signal.len() as f64;
        signal.into_iter().map(move |sample| sample - dc_offset)
    }

    fn normalize(self) -> impl Iterator<Item = f64>
    where
        Self: Sized + Iterator<Item = f64>,
    {
        let signal = self.collect::<Vec<_>>();
        let max_amplitude = signal.iter().fold(f64::NEG_INFINITY, |state, sample| {
            let sample = sample.abs();
            if sample > state { sample } else { state }
        });
        signal.into_iter().map(move |sample| sample / max_amplitude)
    }

    /// Partition the iterator into equally-sized non-overlapping windows. The last window will be
    /// shorter.
    fn window(self, size: usize) -> Windowed<Self>
    where
        Self: Sized + Iterator,
    {
        Windowed::new(size, self)
    }
}

impl<I: Iterator> IteratorExt for I {}
