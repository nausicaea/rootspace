use iterator_ext::IteratorExt;
use numenor::{FromF32Unchecked, IntoF32Unchecked};
use std::{borrow::Borrow, f32::consts::PI};

use crate::util::{to_le_bits, to_nrz};
use crate::{Spec, util::samples_per_bit};

mod discrete_integral;
mod iterator_ext;

/// Modulate a byte-stream onto a carrier wave using Continuous Phase Frequency Shift Keying (CPFSK).
///
/// The implementation was gratefully nabbed from the author of [Not Black Magic](https://web.archive.org/web/20251115022344/https://www.notblackmagic.com/bitsnpieces/afsk/#afsk-modulation).
pub fn modulate<T, I, J, S>(spec: &Spec<S>, data: I) -> impl Iterator<Item = S>
where
    T: Borrow<u8>,
    I: IntoIterator<Item = T, IntoIter = J>,
    J: Iterator<Item = T>,
    S: Copy + FromF32Unchecked + IntoF32Unchecked,
{
    // Amplitude Settings
    let amplitude = (spec.high.into_f32_unchecked() - spec.low.into_f32_unchecked()) / 2.0;

    // Frequency Settings
    // carrier_freq + delta_freq = 2400 Hz; carrier_freq - delta_freq = 1200 Hz
    let carrier_freq = u32::midpoint(spec.mark_frequency, spec.space_frequency);
    let delta_freq = spec.mark_frequency.abs_diff(spec.space_frequency) / 2;
    let sample_rate = spec.sample_rate;
    let carrier_omega = 2.0 * PI * (carrier_freq as f32 / sample_rate as f32);
    let delta_omega = 2.0 * PI * (delta_freq as f32 / sample_rate as f32);

    // Integration settings
    let steps = samples_per_bit(spec.sample_rate as usize, spec.mark_frequency as usize);

    data.into_iter()
        .flat_map(|t| to_le_bits(*t.borrow()))
        .map(|bit| to_nrz(bit))
        .discrete_integral(steps)
        .map(move |(i, m)| {
            let y = modulate_sample(amplitude, carrier_omega, delta_omega, i as f32, m);
            S::from_f32_unchecked(y)
        })
}

fn modulate_sample(amplitude: f32, carrier_omega: f32, delta_omega: f32, t: f32, delta_t: f32) -> f32 {
    amplitude * (carrier_omega * t - delta_omega * delta_t).cos()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hound_test() {
        let spec = Spec::<i8>::with_kcs();
        let mut wav_writer = hound::WavWriter::create(
            "/Users/kitsune/Downloads/cpfsk.i8.wav",
            hound::WavSpec {
                channels: spec.channels,
                sample_rate: spec.sample_rate,
                bits_per_sample: 8,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();

        let data = "Hello, World!".as_bytes();
        for sample in modulate(&spec, data) {
            wav_writer.write_sample(sample).unwrap();
        }
        wav_writer.flush().unwrap();
        wav_writer.finalize().unwrap();
    }
}
