use std::f64::consts::TAU;

use crate::{Spec, util::to_decibel};

/// Apply the Goertzel algorithm (see [`goertzel()`]) for a two-frequency FSK encoding.
pub fn goertzel_with_spec<S>(spec: &Spec<S>, data: &[f64]) -> Output {
    Output {
        mark: goertzel(spec.sample_rate, spec.mark_frequency, data),
        space: goertzel(spec.sample_rate, spec.space_frequency, data),
    }
}

/// Calculate a single discrete Fourier-transform (DFT) term using the [Goertzel
/// algorithm](https://web.archive.org/web/20260120133929/https://en.wikipedia.org/wiki/Goertzel_algorithm).
///
/// Some implementation details were obtained from [Sébastien
/// Piquemal](https://web.archive.org/web/20260115090228/https://gist.githubusercontent.com/sebpiq/4128537/raw/40dcb08387f7c942f6934c2624644d7cb6645633/gistfile1.py)
/// and [Nale
/// Raphael](https://web.archive.org/web/20260120142159/https://github.com/NaleRaphael/goertzel-fft/blob/master/gofft/alg/src/dsp.c).
pub fn goertzel(sample_rate: u32, frequency: u32, data: &[f64]) -> DftTerm {
    let omega = TAU * f64::from(frequency) / f64::from(sample_rate);
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
    use proptest::{prop_assert, proptest};

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
