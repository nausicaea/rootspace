use crate::util::BITMASKS;
use std::borrow::Borrow;
use std::iter::{FusedIterator, repeat_n};

pub fn encode<T, I>(spec: SquareWaveSpec, data: I) -> impl Iterator<Item = i16>
where
    T: Borrow<u8>,
    I: IntoIterator<Item = T>,
{
    let padding_factor = 5;
    padding(spec.sample_rate, padding_factor)
        .chain(data.into_iter().map(|t| *t.borrow()))
        .chain(padding(spec.sample_rate, padding_factor))
        .flat_map(move |byte| encode_byte_le(spec, byte))
}

fn padding(sample_rate: usize, factor: usize) -> impl Iterator<Item = u8> {
    repeat_n(0b1, factor * sample_rate)
}

fn encode_byte_le(spec: SquareWaveSpec, byte: u8) -> impl Iterator<Item = i16> {
    zero_pulse(spec)
        .chain(encode_byte_le_unarmored(spec, byte))
        .chain(one_pulse(spec))
        .chain(one_pulse(spec))
}

fn encode_byte_le_unarmored(spec: SquareWaveSpec, byte: u8) -> impl Iterator<Item = i16> {
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
            low: offset - amplitude,
            high: offset + amplitude,
            period_length,
            num_periods,
            index: 0,
        }
    }

    pub const fn with_spec(spec: SquareWaveSpec) -> Self {
        debug_assert!(spec.target_freq <= (spec.sample_rate >> 1));
        Self::new(
            spec.offset,
            spec.amplitude,
            spec.sample_rate / spec.target_freq,
            spec.num_periods,
        )
    }

    const fn len_internal(&self) -> usize {
        self.period_length * self.num_periods
    }

    /// Return `true` when the square wave amplitude is `1`, and false where it is `0`.
    /// `period_length` must be divisible by two.
    const fn is_high(i: usize, period_length: usize) -> bool {
        debug_assert!(period_length % 2 == 0);
        (2 * i / period_length) % 2 != 0
    }
}

impl Iterator for SquareWave {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let output = if self.index >= self.len_internal() {
            None
        } else if Self::is_high(self.index, self.period_length) {
            Some(self.high)
        } else {
            Some(self.low)
        };

        self.index += 1;
        output
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(remaining) = self.len_internal().checked_sub(self.index) {
            (remaining, Some(remaining))
        } else {
            (0, Some(0))
        }
    }
}

impl ExactSizeIterator for SquareWave {}

impl FusedIterator for SquareWave {}

#[cfg(test)]
mod tests {
    use std::{ops::Range, path::Path};

    use super::*;
    use proptest::{prelude::Just, prop_assert_eq, prop_compose, proptest, sample::select};
    use rstest::{fixture, rstest};

    #[fixture]
    fn test_spec() -> SquareWaveSpec {
        SquareWaveSpec {
            offset: 0x0080,
            amplitude: 0x0080,
            sample_rate: 4,
            target_freq: 2,
            num_periods: 2,
        }
    }

    #[fixture]
    fn kcs_spec() -> SquareWaveSpec {
        SquareWaveSpec {
            offset: 0,
            amplitude: i16::MAX,
            sample_rate: 9600,
            target_freq: 2400,
            num_periods: 8,
        }
    }

    #[rstest]
    #[case(2, &[false, true])]
    #[case(4, &[false, false, true, true])]
    #[case(6, &[false, false, false, true, true, true])]
    #[case(8, &[false, false, false, false, true, true, true, true])]
    fn is_high_period_pattern(#[case] p: usize, #[case] expected: &[bool]) {
        assert_eq!((0..p).map(|i| SquareWave::is_high(i, p)).collect::<Vec<_>>(), expected);
    }

    #[rstest]
    #[case::two_period(4, 2, &[false, true])]
    #[case::four_period(8, 4, &[false, false, true, true])]
    #[case::six_period(12, 6, &[false, false, false, true, true, true])]
    #[case::eight_period(16, 8, &[false, false, false, false, true, true, true, true])]
    fn is_high_period_pattern_repeats(#[case] max_i: usize, #[case] period: usize, #[case] expected: &[bool]) {
        use itertools::Itertools;

        let output = (0..max_i).map(|i| SquareWave::is_high(i, period)).chunks(period);
        for chunk in &output {
            assert_eq!(chunk.collect::<Vec<_>>(), expected);
        }
    }

    prop_compose! {
        fn odd_usize()(i in 1..(usize::MAX / 2)) -> usize {
            i * 2 - 1
        }
    }

    proptest! {
        #[test]
        #[should_panic]
        fn is_high_0_period_panics(i: usize) {
            SquareWave::is_high(i, 0);
        }

        #[test]
        #[should_panic]
        fn is_high_odd_period_panics(i: usize, period_length in odd_usize()) {
            SquareWave::is_high(i, period_length);
        }

        #[test]
        fn is_high_2_period_is_true_for_odd_indices(i in 0..(usize::MAX / 2)) {
            prop_assert_eq!(SquareWave::is_high(i, 2), i % 2 != 0);
        }

        #[test]
        fn is_high_4_period_is_true_in_blocks_of_two(i in 0..(usize::MAX / 2)) {
            prop_assert_eq!(SquareWave::is_high(i, 4), (i / 2) % 2 != 0);
        }

    }

    // #[rstest]
    // fn square_wave_to_wav(kcs_spec: SquareWaveSpec) {
    //     let bits_per_sample = 16;
    //     let sqwave = SquareWave::with_spec(kcs_spec);
    //     let mut wav_writer = hound::WavWriter::create(
    //         Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/simple_square_wave_{}hz_{bits_per_sample}bps_{}samples.wav", kcs_spec.sample_rate, sqwave.len())),
    //         hound::WavSpec { channels: 1, sample_rate: kcs_spec.sample_rate as u32, bits_per_sample, sample_format: hound::SampleFormat::Int }
    //     ).unwrap();

    //     let mut wav_writer_i16 = wav_writer.get_i16_writer(sqwave.len() as u32);
    //     sqwave.for_each(|sample| wav_writer_i16.write_sample(sample));
    //     wav_writer_i16.flush().unwrap();
    //     wav_writer.finalize().unwrap();
    // }

    #[rstest]
    fn square_wave_with_amplitude_offset(test_spec: SquareWaveSpec) {
        let sqwave = SquareWave::with_spec(test_spec).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, &[0x0000, 0x0100, 0x0000, 0x0100]);
    }

    #[test]
    fn square_wave_new_without_offset() {
        let sqwave = SquareWave::new(0, 256, 2, 2).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, &[-0x0100, 0x0100, -0x0100, 0x0100]);
    }

    #[rstest]
    fn square_wave_kcs_spec(kcs_spec: SquareWaveSpec) {
        let sqwave = SquareWave::with_spec(kcs_spec).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 32);
        // Exactly eight periods of a 2400 Hz tone at 9600 Hz sampling rate
        #[rustfmt::skip]
        assert_eq!(sqwave, &[
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX, 
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX,
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX,
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX,
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX, 
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX,
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX,
            -i16::MAX, -i16::MAX, i16::MAX, i16::MAX,
        ]);
    }

    #[rstest]
    fn zero_pulse_is_half_frequency_of_one_pulse_but_same_length(test_spec: SquareWaveSpec) {
        let one = one_pulse(test_spec).collect::<Vec<_>>();
        assert_eq!(one.len(), 4);
        assert_eq!(one, &[0x0000, 0x0100, 0x0000, 0x0100]);
        let zero = zero_pulse(test_spec).collect::<Vec<_>>();
        assert_eq!(zero.len(), 4);
        assert_eq!(zero, &[0x0000, 0x0000, 0x0100, 0x0100]);
    }

    #[rstest]
    fn test_encode_byte_le_unarmored(test_spec: SquareWaveSpec) {
        let samples = encode_byte_le_unarmored(test_spec, 0x01).collect::<Vec<_>>();
        assert_eq!(samples.len(), 4 * 8);
        #[rustfmt::skip]
        assert_eq!(samples, &[
            0x0000, 0x0100, 0x0000, 0x0100, // 0b0000_0001 * 1
            0x0000, 0x0000, 0x0100, 0x0100, // 0b0000_0010 * 0
            0x0000, 0x0000, 0x0100, 0x0100, // 0b0000_0100 * 0
            0x0000, 0x0000, 0x0100, 0x0100, // 0b0000_1000 * 0
            0x0000, 0x0000, 0x0100, 0x0100, // 0b0001_0000 * 0
            0x0000, 0x0000, 0x0100, 0x0100, // 0b0010_0000 * 0
            0x0000, 0x0000, 0x0100, 0x0100, // 0b0100_0000 * 0
            0x0000, 0x0000, 0x0100, 0x0100, // 0b1000_0000 * 0
        ]);
    }

    prop_compose! {
        fn powers_of_two_u8()(p in 0_u8..7) -> u8 {
            2 << p
        }
    }

    fn mismatching(src: Range<u8>, el: u8) -> Vec<u8> {
        src.filter(move |&e| e != el).collect()
    }

    prop_compose! {
        fn mismatching_powers_of_two_u8()(p1 in 0_u8..7)(p1 in Just(p1), p2 in select(mismatching(0_u8..7, p1))) -> (u8, u8) {
            (2 << p1, 2 << p2)
        }
    }

    proptest! {
        #[test]
        fn encode_bit_encode_same_is_always_0b1_encoded(bit in powers_of_two_u8()) {
            let samples = encode_bit(
                test_spec(),
                bit,
                bit,
            ).collect::<Vec<_>>();

            prop_assert_eq!(samples.len(), 4);
            prop_assert_eq!(samples, &[0x0000, 0x0100, 0x0000, 0x0100]);
        }

        #[test]
        fn encode_bit_encode_mismatching_is_always_0b0_encoded((b1, b2) in mismatching_powers_of_two_u8()) {
            let samples = encode_bit(
                test_spec(),
                b1,
                b2,
            ).collect::<Vec<_>>();

            prop_assert_eq!(samples.len(), 4);
            prop_assert_eq!(samples, &[0x0000, 0x0000, 0x0100, 0x0100]);
        }
    }
}
