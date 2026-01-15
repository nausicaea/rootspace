/// See [`IteratorExt::discrete_integral()`](super::iterator_ext::IteratorExt::discrete_integral) for more information.
pub struct DiscreteIntegral<I> {
    source: I,
    samples_per_bit: usize,
    index: usize,
    k_prev: usize,
    k: usize,
    sample_k_prev: f32,
    sample_k: f32,
    m: f32,
}

impl<I> DiscreteIntegral<I> {
    pub(crate) const fn new(samples_per_bit: usize, source: I) -> Self {
        Self {
            source,
            samples_per_bit,
            k_prev: 0,
            k: 0,
            sample_k_prev: 0.0,
            sample_k: 0.0,
            index: 0,
            m: 0.0,
        }
    }
}

impl<I> Iterator for DiscreteIntegral<I>
where
    I: Iterator<Item = i8>,
{
    type Item = (usize, f32);

    fn next(&mut self) -> Option<Self::Item> {
        let k_prev = self.index / self.samples_per_bit;
        let k = (self.index + 1) / self.samples_per_bit;

        match ((k_prev - self.k_prev) > 0, (k - self.k) > 0) {
            // The very first iteration has special handling because it needs to set up the sample
            // cache.
            (false, false) if self.index == 0 => {
                self.sample_k_prev = self.source.next()? as f32;
                self.sample_k = self.sample_k_prev;
            }
            // Because both indices remain unchanged, no updates to the samples are needed.
            (false, false) => (),
            // New data needs to be fetched because k changes, but sample_k_prev remains unchanged.
            (false, true) => {
                self.sample_k = self.source.next()? as f32;
            }
            // No new data is fetched, but sample_k_prev is overwritten by sample_k
            (true, false) => {
                self.sample_k_prev = self.sample_k;
            }
            // Both indices change, and thus we need to overwrite sample_k_prev _and_ fetch new
            // data. I don't expect this case to ever happen.
            (true, true) => {
                self.sample_k_prev = self.sample_k;
                self.sample_k = self.source.next()? as f32;
            }
        }
        self.k_prev = k_prev;
        self.k = k;
        self.m += (self.sample_k_prev + self.sample_k) / 2.0;
        let output = Some((self.index, self.m));
        self.index += 1;
        output
    }
}

#[cfg(test)]
mod tests {
    use super::DiscreteIntegral;
    use rstest::rstest;

    #[rstest]
    #[case(32, &[1, -1, -1, -1, -1, -1, -1, -1])]
    fn discrete_integral(#[case] samples_per_bit: usize, #[case] data: &[i8]) {
        /// This follows the original implementation on https://notblackmagic.com/bitsnpieces/afsk/
        fn original_integral(steps: usize, data: &[i8]) -> Vec<f32> {
            let n = data.len() * steps;
            let mut output = Vec::with_capacity(n);
            let mut m = 0.0;
            for i in 2..=n {
                let index = (i as f32 / steps as f32).ceil() as usize;
                let index_prev = ((i - 1) as f32 / steps as f32).ceil() as usize;

                m += (data[index_prev - 1] as f32 + data[index - 1] as f32) / 2.0;
                eprintln!(
                    "i={i}, data[{index_prev}]={}, data[{index}]={}, m={m}",
                    data[index_prev - 1],
                    data[index - 1]
                );
                output.push(m);
            }
            output
        }

        assert_eq!(
            DiscreteIntegral::new(samples_per_bit, data.into_iter().copied())
                .map(|(_, m)| m)
                .collect::<Vec<_>>(),
            original_integral(samples_per_bit, data),
        );
    }
}
