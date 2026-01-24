use iterator_ext::IteratorExt;
use std::f64::consts::TAU;

use crate::Spec;
use crate::cpfsk::goertzel::goertzel_with_spec;
use crate::util::to_nrz;
use numenor::FromF64Unchecked;

mod goertzel;
pub mod integrate;
pub mod interpolate;
pub mod iterator_ext;
pub mod windowed;

/// Modulate a bitstream onto a carrier wave using Continuous Phase Frequency Shift Keying (CPFSK).
///
/// The implementation was gratefully nabbed from the author of [Not Black
/// Magic](https://web.archive.org/web/20251115022344/https://www.notblackmagic.com/bitsnpieces/afsk/#afsk-modulation).
pub fn modulate<I, S>(spec: &Spec<S>, data: I) -> impl Iterator<Item = S>
where
    I: Iterator<Item = bool>,
    S: Copy + Into<f64> + FromF64Unchecked,
{
    // Amplitude Settings
    let amplitude = (spec.high.into() - spec.low.into()) / 2.0;

    // Frequency Settings
    // carrier_freq + delta_freq = 2400 Hz; carrier_freq - delta_freq = 1200 Hz
    let carrier_freq = u32::midpoint(spec.mark_frequency, spec.space_frequency);
    let delta_freq = spec.mark_frequency.abs_diff(spec.space_frequency) / 2;
    let sample_rate = spec.sample_rate;
    let carrier_omega = TAU * (f64::from(carrier_freq) / f64::from(sample_rate));
    let delta_omega = TAU * (f64::from(delta_freq) / f64::from(sample_rate));

    // Integration settings
    let steps = spec.bit_width();

    data.map(to_nrz)
        .interpolate(steps)
        .integrate()
        .enumerate()
        .map(move |(i, m)| {
            let y = modulate_sample(amplitude, carrier_omega, delta_omega, i as f64, m);
            S::from_f64_unchecked(y)
        })
}

/// Demodulate a bitstream from a FSK-encoded waveform using the Goertzel Algorithm
pub fn demodulate<I, S>(spec: &Spec<S>, data: I) -> impl Iterator<Item = bool>
where
    I: Iterator<Item = S>,
    S: Copy + Into<f64>,
{
    let window_size = spec.bit_width();

    data.map(Into::into)
        // Preprocess the signal
        .center()
        .normalize()
        .window(window_size)
        .map(move |mut signal| {
            // Fill underlength windows
            while signal.len() < window_size {
                signal.push(0.0);
            }

            // Apply the Goertzel algorithm on the window
            goertzel_with_spec(spec, &signal).rel_power()
        })
        .scan(false, move |state, power_db| Some(classify_with_hysteresis(state, spec, power_db)))
}

fn modulate_sample(amplitude: f64, carrier_omega: f64, delta_omega: f64, t: f64, delta_t: f64) -> f64 {
    amplitude * carrier_omega.mul_add(t, delta_omega * delta_t).cos()
}

const fn classify_with_hysteresis<S>(state: &mut bool, spec: &Spec<S>, power_db: f64) -> bool {
    if *state && power_db < spec.space_power_threshold_db {
        *state = false;
    } else if !*state && power_db > spec.mark_power_threshold_db {
        *state = true;
    }

    *state
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::rstest;

    #[rstest]
    fn modulate_kcs_single_bit_output_expectations(#[values(false, true)] bit: bool) {
        let spec = Spec::<i16>::with_kcs();
        let waveform: Vec<_> = modulate(&spec, [bit].into_iter()).collect();
        assert_eq!(waveform.len(), spec.bit_width());
    }

    #[test]
    fn roundtrip_modulation() {
        let input = [false];
        let spec = Spec::<i16>::with_kcs();
        let window_size = spec.bit_width();
        let waveform: Vec<_> = modulate(&spec, input.into_iter()).collect();
        dbg!(&waveform);
        assert_eq!(waveform.len(), input.len() * window_size);

        let output: Vec<_> = demodulate(&spec, waveform.iter().copied()).collect();
        assert_eq!(output, &input, "expected {input:?}, got {output:?}");
    }
}
