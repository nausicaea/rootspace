use std::f64::consts::TAU;

use crate::{Spec, util::to_decibel};

/// Apply the Goertzel algorithm (see [`goertzel()`]) for a two-frequency FSK encoding.
pub fn goertzel_with_spec<S>(spec: &Spec<S>, data: &[f64]) -> Output {
    Output {
        mark: goertzel(spec.sample_rate as usize, spec.mark_frequency as usize, data),
        space: goertzel(spec.sample_rate as usize, spec.space_frequency as usize, data),
    }
}

/// Calculate a single discrete Fourier-transform (DFT) term using the [Goertzel
/// algorithm](https://web.archive.org/web/20260120133929/https://en.wikipedia.org/wiki/Goertzel_algorithm).
///
/// Some implementation details were obtained from [Sébastien
/// Piquemal](https://web.archive.org/web/20260115090228/https://gist.githubusercontent.com/sebpiq/4128537/raw/40dcb08387f7c942f6934c2624644d7cb6645633/gistfile1.py)
/// and [Nale
/// Raphael](https://web.archive.org/web/20260120142159/https://github.com/NaleRaphael/goertzel-fft/blob/master/gofft/alg/src/dsp.c).
pub fn goertzel(sample_rate: usize, frequency: usize, data: &[f64]) -> DftTerm {
    let window_size = data.len();
    let k = k_term(sample_rate, frequency, window_size);
    let omega = TAU * k / (window_size as f64);
    let omega_real = 2.0 * omega.cos();
    let omega_imag = omega.sin();

    let (d1, d2) = data
        .iter()
        .fold((0.0, 0.0), move |state, &sample| filter_pass(state, sample, omega_real));

    DftTerm {
        omega_real,
        omega_imag,
        d1,
        d2,
    }
}

/// Calculate the
/// [`KTerm`](https://web.archive.org/web/20260120133929/https://en.wikipedia.org/wiki/Goertzel_algorithm#DFT_computations),
/// or frequency bin, for the Goertzel algorithm.
fn k_term(sample_rate: usize, frequency: usize, window_size: usize) -> f64 {
    ((window_size as f64 * frequency as f64) / (sample_rate as f64)).round()
}

/// Calculate a single filter pass for the Goertzel algorithm
fn filter_pass(state: (f64, f64), sample: f64, omega_real: f64) -> (f64, f64) {
    let (d1, d2) = state;
    let y = sample + omega_real * d1 - d2;
    (y, d1)
}

#[derive(Debug)]
pub struct Output {
    pub mark: DftTerm,
    pub space: DftTerm,
}

impl Output {
    pub fn rel_power(&self) -> f64 {
        to_decibel(self.mark.power(), self.space.power())
    }
}

#[derive(Debug)]
pub struct DftTerm {
    pub omega_real: f64,
    pub omega_imag: f64,
    pub d1: f64,
    pub d2: f64,
}

impl DftTerm {
    pub fn power(&self) -> f64 {
        self.d2.powi(2) + self.d1.powi(2) - self.omega_real * self.d1 * self.d2
    }

    #[allow(dead_code)]
    pub fn real(&self) -> f64 {
        0.5 * self.omega_real * self.d1 - self.d2
    }

    #[allow(dead_code)]
    pub fn imag(&self) -> f64 {
        self.omega_imag * self.d1
    }
}

#[cfg(test)]
mod tests {
    use crate::util::{samples_per_bit, to_decibel};

    use super::*;
    use proptest::{prop_assert, prop_assert_eq, proptest};
    use rstest::rstest;

    #[rstest]
    #[case::kcs_space_frequency(1200, 9600, 32, 4.0)]
    #[case::kcs_mark_frequency(2400, 9600, 32, 8.0)]
    fn k_term_kcs(#[case] f: usize, #[case] s: usize, #[case] w: usize, #[case] expected: f64) {
        assert_eq!(k_term(s, f, w), expected)
    }

    fn goertzel_power<S>(spec: &Spec<S>, freq: usize) -> f64 {
        let window_size = samples_per_bit(
            spec.sample_rate as usize,
            spec.mark_frequency as usize,
            spec.mark_num_periods,
        );
        let time: Vec<_> = (0..(spec.sample_rate as usize))
            .into_iter()
            .take(window_size)
            .map(|i| i as f64 / spec.sample_rate as f64)
            .collect();
        let amplitude = i8::MAX as f64;
        let signal: Vec<_> = time.iter().map(|t| amplitude * (TAU * freq as f64 * t).sin()).collect();
        let output = goertzel_with_spec(&spec, &signal);
        to_decibel(output.mark.power(), output.space.power())
    }

    proptest! {
        /// Test for integer division errors
        #[test]
        fn k_term_integer_division(s in 4800_usize..=44100) {
            let f = 2400;
            let c = 8;
            let w = samples_per_bit(s, f, c);
            let k = k_term(s, f, w);
            prop_assert_eq!(k, (0.5 + ((w as f64 * f as f64) / s as f64)).floor());
        }

        #[test]
        fn goertzel_power_space_frequencies(f in 1195_usize..=1205) {
            let spec = Spec::<i8>::with_kcs();
            let power_ratio = goertzel_power(&spec, f);
            prop_assert!(power_ratio <= spec.space_power_threshold_db, "got {power_ratio}, expected <= {}", spec.space_power_threshold_db);
        }

        #[test]
        fn goertzel_power_mark_frequencies(f in 2395_usize..=2405) {
            let spec = Spec::<i8>::with_kcs();
            let power_ratio = goertzel_power(&spec, f);
            prop_assert!(power_ratio >= spec.mark_power_threshold_db, "got {power_ratio}, expected >= {}", spec.mark_power_threshold_db);
        }
    }
}
