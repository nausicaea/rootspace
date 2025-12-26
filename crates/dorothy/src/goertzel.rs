use std::borrow::Borrow;

use crate::shared::above_nyquist_limit;

///
/// # Assumptions
///
/// 1. Real-valued input data
/// 2. The block size is chosen sensibly (i.e. multiples of sampling rate / target frequency).
#[derive(Debug)]
pub struct Goertzel {
    omega_cos: f32,
    omega_sin: f32,
    coeff0: f32,
    qnm2: f32,
    qnm1: f32,
}

impl Goertzel {
    pub fn new(p: &Parameters) -> Self {
        let f = p.target_frequency as f32;
        let s = p.sampling_rate as f32;
        let n = p.block_size as f32;

        let k = (0.5 + (n * f) / s).floor();
        let omega = std::f32::consts::TAU / n * k;
        let omega_cos = omega.cos();
        let omega_sin = omega.sin();
        let coeff0 = 2.0 * omega_cos;

        Self {
            omega_cos,
            omega_sin,
            coeff0,
            qnm2: 0.0,
            qnm1: 0.0,
        }
    }

    pub fn add_block(mut self, samples: impl IntoIterator<Item = impl Borrow<f32>>) -> Self {
        self.qnm2 = 0.0;
        self.qnm1 = 0.0;

        for sample in samples {
            let qn = self.coeff0 * self.qnm1 - self.qnm2 + sample.borrow();
            self.qnm2 = self.qnm1;
            self.qnm1 = qn;
        }

        self
    }

    pub fn finish(self) -> Tone {
        Tone(self)
    }
}

#[derive(Debug)]
pub struct Tone(Goertzel);

impl Tone {
    pub fn phase(&self) -> (f32, f32) {
        (
            self.0.qnm1 * self.0.omega_cos - self.0.qnm2,
            self.0.qnm1 * self.0.omega_sin,
        )
    }

    pub fn power(&self) -> f32 {
        self.0.qnm1.powi(2) + self.0.qnm2.powi(2) - (self.0.coeff0 * self.0.qnm1 * self.0.qnm2)
    }
}

#[derive(Debug, Clone)]
pub struct Parameters {
    target_frequency: u32,
    sampling_rate: u32,
    block_size: u32,
}

impl Parameters {
    pub fn new(target_frequency: u32, sampling_rate: u32, block_size: u32) -> Result<Self, Error> {
        if !above_nyquist_limit(target_frequency, sampling_rate) {
            return Err(Error::NyquistViolation);
        }

        Ok(Parameters {
            target_frequency,
            sampling_rate,
            block_size,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The sampling rate must be at least twice as large as the target frequency")]
    NyquistViolation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use proptest::{prelude::{Just, Strategy}, prop_assert, prop_compose, proptest};

    #[fixture]
    fn default_parameters() -> Parameters {
        Parameters::new(440, 4000, 100).unwrap()
    }

    #[fixture]
    fn sine_wave(default_parameters: Parameters) -> Vec<f32> {
        (0..default_parameters.sampling_rate)
            .map(|t| {
                let time = t as f32 / default_parameters.sampling_rate as f32;
                (std::f32::consts::TAU * default_parameters.target_frequency as f32 * time).sin()
            })
            .collect()
    }

    #[fixture]
    fn spot_on_tone(default_parameters: Parameters, sine_wave: Vec<f32>) -> Tone {
        Goertzel::new(&default_parameters)
            .add_block(&sine_wave)
            .finish()
    }

    #[test]
    #[should_panic(expected = "NyquistViolation")]
    fn sampling_rate_must_be_at_least_twice_the_frequency() {
        Parameters::new(440, 600, 2).unwrap();
    }

    #[rstest]
    fn add_blocks_builder_pattern(default_parameters: Parameters) {
        let _: Goertzel = Goertzel::new(&default_parameters)
            .add_block(&[0.0, 1.0, 2.0, 3.0])
            .add_block(&[4.0, 5.0, 6.0, 7.0]);
    }

    #[rstest]
    fn spot_on_detection(spot_on_tone: Tone) {
        assert!(spot_on_tone.power() > 0.0);
    }

    #[test]
    fn power_cannot_be_negative() {
        let target_freq = 2;
        let sampling_rate = 4000;
        let block_size = 100;


        let params = Parameters::new(target_freq, sampling_rate, block_size).unwrap();
        let tone = Goertzel::new(&params)
            .add_block(&sine_wave(params))
            .finish();

        assert!(tone.power() > 0.0, "P={} !> 0", tone.power());
    }

    prop_compose! {
        fn off_target_frequencies(f: u32, s: u32, d: u32)(otf in (1_u32..(f - d)).prop_union((f + d + 1)..(s >> 2))) -> u32 {
            otf
        }
    }

    prop_compose! {
        fn on_target_frequencies(f: u32, d: u32)(otf in (f - d)..(f + d)) -> u32 {
            otf
        }
    }

    prop_compose! {
        fn band_pass(width: u32, sampling_rate: u32)(f in (1 + width)..((sampling_rate >> 2) - width))(f in Just(f), freq_range in (f - width)..(f + width)) -> (u32, u32) {
            (f, freq_range)
        }
    }

    proptest! {
        #[test]
        fn power_is_very_small_for_off_target_frequencies(f in off_target_frequencies(440, 4000, 20)) {
            let p = default_parameters();
            let tone = Goertzel::new(&Parameters::new(f, p.sampling_rate, p.block_size).unwrap())
                .add_block(&sine_wave(p))
                .finish();
            prop_assert!(tone.power() < 0.00005, "P={}", tone.power())
        }

        #[test]
        fn power_is_very_high_for_on_target_frequencies(f in on_target_frequencies(440, 20)) {
            let p = default_parameters();
            let tone = Goertzel::new(&Parameters::new(f, p.sampling_rate, p.block_size).unwrap())
                .add_block(&sine_wave(p))
                .finish();
            prop_assert!(tone.power() > 999999.0, "P={}", tone.power())
        }
        
        #[test]
        fn power_is_very_high_for_on_target_frequencies_2((target_freq, test_freq) in band_pass(1, 4000)) {
            let p = default_parameters();
            let tone = Goertzel::new(&Parameters::new(test_freq, p.sampling_rate, p.block_size).unwrap())
                .add_block(&sine_wave(Parameters::new(target_freq, p.sampling_rate, p.block_size).unwrap()))
                .finish();
            prop_assert!(tone.power() > 999999.0, "target_freq={}, test_freq={}, P={}", target_freq, test_freq, tone.power())
        }
    }

}
