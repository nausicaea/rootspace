pub struct Interpolate<I> {
    factor: f64,
    source: I,
    i: usize,
    k_prev: usize,
    k: usize,
    sample_k_prev: f64,
    sample_k: f64,
}

impl<I> Interpolate<I> {
    pub fn new(factor: usize, source: I) -> Self {
        Self {
            factor: factor as f64,
            source,
            i: 0,
            k_prev: 0,
            k: 0,
            sample_k_prev: 0.0,
            sample_k: 0.0,
        }
    }
}

impl<I> Iterator for Interpolate<I>
where
    I: Iterator<Item = f64>,
{
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == 0 {
            self.i += 1;
            return Some(0.0);
        }
        let k_prev = (self.i as f64 / self.factor).ceil() as usize - 1;
        let k = ((self.i + 1) as f64 / self.factor).ceil() as usize - 1;
        match ((k_prev - self.k_prev) > 0, (k - self.k) > 0) {
            // The very first iteration has special handling because it needs to set up the sample
            // cache.
            (false, false) if self.i == 1 => {
                self.sample_k_prev = self.source.next()?;
                self.sample_k = self.sample_k_prev;
            }
            // Because both indices remain unchanged, no updates to the samples are needed.
            (false, false) => (),
            // New data needs to be fetched because k changes, but sample_k_prev remains unchanged.
            (false, true) => {
                self.sample_k = self.source.next()?;
            }
            // No new data is fetched, but sample_k_prev is overwritten by sample_k
            (true, false) => {
                self.sample_k_prev = self.sample_k;
            }
            // Both indices change, and thus we need to overwrite sample_k_prev _and_ fetch new
            // data. I don't expect this case to ever happen.
            (true, true) => {
                self.sample_k_prev = self.sample_k;
                self.sample_k = self.source.next()?;
            }
        }
        self.k_prev = k_prev;
        self.k = k;
        let output = Some(f64::midpoint(self.sample_k_prev, self.sample_k));
        self.i += 1;
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This follows the original implementation on https://notblackmagic.com/bitsnpieces/afsk/ The
    /// important difference here is that the interpolation code was extracted from the original
    /// loop that manages a lot of unrelated state, and calculates a cumulative sum / integral.
    fn interpolate_orig(steps: usize, data: &[f64]) -> Vec<f64> {
        let n = steps * data.len();
        let mut output = vec![0.0];
        for i in 1..n {
            let index = ((i + 1) as f64 / steps as f64).ceil() as usize - 1;
            let index_prev = (i as f64 / steps as f64).ceil() as usize - 1;
            output.push(f64::midpoint(data[index], data[index_prev]));
        }

        output
    }

    #[test]
    fn interpolate_orig_2_0_4() {
        assert_eq!(
            interpolate_orig(2, &[0.0, 1.0, 2.0, 3.0]),
            &[0.0, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0],
        );
    }

    #[test]
    fn interpolation_iterator_matches_expectations() {
        let a = interpolate_orig(2, &[0.0, 1.0, 2.0, 3.0]);
        let b = Interpolate::new(2, [0.0, 1.0, 2.0, 3.0].into_iter()).collect::<Vec<_>>();
        assert_eq!(a, b);
    }
}
