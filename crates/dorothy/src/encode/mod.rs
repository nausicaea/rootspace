use std::array::IntoIter;
use crate::util::BITMASKS;
use std::borrow::Borrow;
use std::iter::{repeat_n, FusedIterator};

pub fn encode<T, I>(
    spec: SquareWaveSpec,
    data: I,
) -> impl Iterator<Item = i16>
where
    T: Borrow<u8>,
    I: IntoIterator<Item = T>,
{
    let padding_factor = 5;
    padding(spec.sample_rate, padding_factor)
        .chain(data.into_iter().map(|t| *t.borrow()))
        .chain(padding(spec.sample_rate, padding_factor))
        .flat_map(move |byte| encode_byte(spec, byte))
}

fn padding(sample_rate: usize, factor: usize) -> impl Iterator<Item = u8> {
    repeat_n(0b1, factor * sample_rate)
}

fn encode_byte(spec: SquareWaveSpec, byte: u8) -> impl Iterator<Item = i16> {
    zero_pulse(spec)
        .chain(encode_byte_unarmored(spec, byte))
        .chain(one_pulse(spec))
        .chain(one_pulse(spec))
}

fn encode_byte_unarmored(spec: SquareWaveSpec, byte: u8) -> impl Iterator<Item = i16> {
    BITMASKS.into_iter().flat_map(move |mask| encode_bit(spec, mask, byte))
}

const fn encode_bit(spec: SquareWaveSpec, mask: u8, byte: u8) -> SquareWave {
    if byte & mask != 0 {
        one_pulse(spec)
    } else {
        zero_pulse(spec)
    }
}

const fn one_pulse(spec: SquareWaveSpec) -> SquareWave {
    SquareWave::with_spec(spec)
}

const fn zero_pulse(spec: SquareWaveSpec) -> SquareWave {
    SquareWave::with_spec(SquareWaveSpec {
        target_freq: spec.target_freq / 2,
        num_periods: spec.num_periods / 2,
        ..spec
    })
}

#[derive(Debug, Clone, Copy)]
pub struct SquareWaveSpec {
    pub offset: i16,
    pub amplitude: i16,
    pub sample_rate: usize,
    pub target_freq: usize,
    pub num_periods: usize,
}

#[derive(Debug, Clone)]
pub struct SquareWave {
    low: i16,
    high: i16,
    period_length: usize,
    num_periods: usize,
    index: usize,
}

impl SquareWave {
    pub const fn new(offset: i16, amplitude: i16, period_length: usize, num_periods: usize) -> Self {
        debug_assert!(period_length % 2 == 0);
        Self {
            low: offset - amplitude / 2,
            high: offset + amplitude / 2,
            period_length,
            num_periods,
            index: 0,
        }
    }

    pub const fn with_spec(spec: SquareWaveSpec) -> Self {
        debug_assert!(spec.target_freq <= (spec.sample_rate >> 1));
        Self::new(spec.offset, spec.amplitude, spec.sample_rate / spec.target_freq, spec.num_periods)
    }
}

impl Iterator for SquareWave {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let output = if self.index >= self.num_periods * self.period_length {
            None
        } else if square_wave_predicate(self.index, self.period_length) {
            Some(self.low)
        } else {
            Some(self.high)
        };

        self.index += 1;
        output
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(remaining) = (self.num_periods * self.period_length).checked_sub(self.index) {
            (remaining, Some(remaining))
        } else {
            (0, Some(0))
        }
    }
}

impl ExactSizeIterator for SquareWave {}

impl FusedIterator for SquareWave {}

const fn square_wave_predicate(i: usize, period_length: usize) -> bool {
    debug_assert!(period_length % 2 == 0);
    (i / (period_length / 2)) % period_length == 0
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest]
    #[case(0, 2, true)]
    #[case(1, 2, false)]
    #[case(2, 2, true)]
    #[case(3, 2, false)]
    #[case(0, 4, true)]
    #[case(1, 4, true)]
    #[case(2, 4, false)]
    #[case(3, 4, false)]
    fn square_wave_predicate_expectations(
        #[case] i: usize,
        #[case] period_length: usize,
        #[case] expected: bool,
    ) {
        assert_eq!(square_wave_predicate(i, period_length), expected);
    }

    #[test]
    fn square_wave_with_amplitude_offset() {
        let sqwave = SquareWave::new(128, 128, 2, 2).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, &[0x0040, 0x00C0, 0x0040, 0x00C0]);
    }

    #[test]
    fn square_wave_new_without_offset() {
        let sqwave = SquareWave::new(0, 256, 2, 2).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, &[-0x0080, 0x0080, -0x0080, 0x0080]);
    }

    #[test]
    fn zero_pulse_is_half_frequency_of_one_pulse_but_same_length() {
        let spec = SquareWaveSpec {
            offset: 0x0080,
            amplitude: 0x0080,
            sample_rate: 4,
            target_freq: 2,
            num_periods: 2,
        };

        let one = one_pulse(spec).collect::<Vec<_>>();
        assert_eq!(one.len(), 4);
        assert_eq!(one, &[0x0040, 0x00C0, 0x0040, 0x00C0]);
        let zero = zero_pulse(spec).collect::<Vec<_>>();
        assert_eq!(zero.len(), 4);
        assert_eq!(zero, &[0x0040, 0x0040, 0x00C0, 0x00C0]);
    }

    #[test]
    fn asdasd() {
        let samples = encode_bit(
            SquareWaveSpec {
                offset: 128,
                amplitude: 128,
                sample_rate: 4,
                target_freq: 2,
                num_periods: 1,
            },
            0x01,
            0xff,
        ).collect::<Vec<_>>();

        assert_eq!(samples.len(), 2);
        assert_eq!(samples, &[0x0040, 0x00C0]);
    }
}