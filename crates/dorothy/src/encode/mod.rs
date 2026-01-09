use crate::util::BITMASKS;
use std::borrow::Borrow;
use std::iter::FusedIterator;

pub fn encode<T, I>(spec: SquareWaveSpec, padding_factor: usize, data: I) -> impl Iterator<Item = i8>
where
    T: Borrow<u8>,
    I: IntoIterator<Item = T>,
{
    padding(spec, padding_factor)
        .chain(
            data.into_iter()
                .map(|t| *t.borrow())
                .flat_map(move |byte| encode_byte_le(spec, byte)),
        )
        .chain(padding(spec, padding_factor))
}

const fn padding(spec: SquareWaveSpec, factor: usize) -> SquareWave {
    SquareWave::with_spec(SquareWaveSpec {
        num_periods: factor * (spec.target_freq as usize),
        ..spec
    })
}

fn encode_byte_le(spec: SquareWaveSpec, byte: u8) -> impl Iterator<Item = i8> {
    SquareWave::zero_pulse(spec)
        .chain(encode_byte_le_unarmored(spec, byte))
        .chain(SquareWave::one_pulse(spec))
        .chain(SquareWave::one_pulse(spec))
}

fn encode_byte_le_unarmored(spec: SquareWaveSpec, byte: u8) -> impl Iterator<Item = i8> {
    BITMASKS.into_iter().flat_map(move |mask| encode_bit(spec, mask, byte))
}

const fn encode_bit(spec: SquareWaveSpec, mask: u8, byte: u8) -> SquareWave {
    if byte & mask != 0 {
        SquareWave::one_pulse(spec)
    } else {
        SquareWave::zero_pulse(spec)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SquareWaveSpec {
    pub offset: i8,
    pub amplitude: i8,
    pub sample_rate: usize,
    pub target_freq: usize,
    pub num_periods: usize,
}

#[derive(Debug, Clone)]
pub struct SquareWave {
    low: i8,
    high: i8,
    period_length: usize,
    num_periods: usize,
    index: usize,
}

impl SquareWave {
    #[must_use]
    pub const fn new(offset: i8, amplitude: i8, period_length: usize, num_periods: usize) -> Self {
        Self {
            low: offset - amplitude,
            high: offset + amplitude,
            period_length,
            num_periods,
            index: 0,
        }
    }

    #[must_use]
    pub const fn with_spec(spec: SquareWaveSpec) -> Self {
        debug_assert!(spec.target_freq <= (spec.sample_rate >> 1));
        Self::new(
            spec.offset,
            spec.amplitude,
            spec.sample_rate / spec.target_freq,
            spec.num_periods,
        )
    }

    #[must_use]
    pub const fn one_pulse(spec: SquareWaveSpec) -> Self {
        Self::with_spec(spec)
    }

    #[must_use]
    pub const fn zero_pulse(spec: SquareWaveSpec) -> Self {
        Self::with_spec(SquareWaveSpec {
            target_freq: spec.target_freq / 2,
            num_periods: spec.num_periods / 2,
            ..spec
        })
    }

    const fn len_internal(&self) -> usize {
        self.period_length * self.num_periods
    }

    /// Return `true` when the square wave amplitude is `1`, and false where it is `0`.
    const fn is_high(i: usize, period_length: usize) -> bool {
        !(2 * i / period_length).is_multiple_of(2)
    }
}

impl Iterator for SquareWave {
    type Item = i8;

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

    use crate::util::tests::{mismatching_powers_of_two_u8, powers_of_two_u8, sr_and_tf};

    use super::*;
    use proptest::{prop_assert_eq, proptest};
    use rstest::{fixture, rstest};

    type Spec = (SquareWaveSpec, &'static [i8], &'static [i8]);

    /// Returns a tuple with the square wave specification, and static values for 0b01 encoded and
    /// 0b00 encoded in that order.
    #[fixture]
    fn test_spec() -> Spec {
        (
            SquareWaveSpec {
                offset: i8::MAX / 2,
                amplitude: i8::MAX / 2,
                sample_rate: 4,
                target_freq: 2,
                num_periods: 2,
            },
            &[0x00, 0x7E, 0x00, 0x7E], // 0b1 encoded
            &[0x00, 0x00, 0x7E, 0x7E], // 0b0 encoded
        )
    }

    /// Returns a tuple with the square wave specification, and static values for 0b01 encoded and
    /// 0b00 encoded in that order.
    #[fixture]
    fn kcs_spec() -> Spec {
        (
            SquareWaveSpec {
                offset: 0,
                amplitude: i8::MAX,
                sample_rate: 9600,
                target_freq: 2400,
                num_periods: 8,
            },
            &[
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
            ],
            &[
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                -i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
                i8::MAX,
            ],
        )
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

    proptest! {
        #[test]
        fn is_high_2_period_is_true_for_odd_indices(i in 0..(usize::MAX / 2)) {
            prop_assert_eq!(SquareWave::is_high(i, 2), !i.is_multiple_of(2));
        }

        #[test]
        fn is_high_4_period_is_true_in_blocks_of_two(i in 0..(usize::MAX / 2)) {
            prop_assert_eq!(SquareWave::is_high(i, 4), !(i / 2).is_multiple_of(2));
        }

    }

    #[rstest]
    fn square_wave_with_amplitude_offset(test_spec: Spec) {
        let sqwave = SquareWave::with_spec(test_spec.0).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, test_spec.1);
    }

    #[test]
    fn square_wave_new_without_offset() {
        let sqwave = SquareWave::new(0, i8::MAX, 2, 2).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, &[-i8::MAX, i8::MAX, -i8::MAX, i8::MAX]);
    }

    #[rstest]
    fn square_wave_kcs_spec(kcs_spec: Spec) {
        let sqwave = SquareWave::with_spec(kcs_spec.0).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 32);
        // Exactly eight periods of a 2400 Hz tone at 9600 Hz sampling rate
        assert_eq!(sqwave, kcs_spec.1);
    }

    #[rstest]
    fn zero_pulse_and_one_pulse_are_equal_length(test_spec: Spec) {
        let one = SquareWave::one_pulse(test_spec.0).collect::<Vec<_>>();
        assert_eq!(one.len(), 4);
        assert_eq!(one, test_spec.1);
        let zero = SquareWave::zero_pulse(test_spec.0).collect::<Vec<_>>();
        assert_eq!(zero.len(), 4);
        assert_eq!(zero, test_spec.2);
    }

    #[rstest]
    fn kcs_zero_pulse_and_one_pulse_are_equal_length(kcs_spec: Spec) {
        let one = SquareWave::one_pulse(kcs_spec.0).collect::<Vec<_>>();
        assert_eq!(one.len(), 32);
        assert_eq!(one, kcs_spec.1);
        let zero = SquareWave::zero_pulse(kcs_spec.0).collect::<Vec<_>>();
        assert_eq!(zero.len(), 32);
        assert_eq!(zero, kcs_spec.2);
    }

    #[rstest]
    fn encode_byte_le_unarmored_0x01(test_spec: Spec) {
        let samples = encode_byte_le_unarmored(test_spec.0, 0x01).collect::<Vec<_>>();
        assert_eq!(samples.len(), 32);
        #[rustfmt::skip]
        assert_eq!(samples, [
            test_spec.1, // 0b0000_0001 * 1
            test_spec.2, // 0b0000_0010 * 0
            test_spec.2, // 0b0000_0100 * 0
            test_spec.2, // 0b0000_1000 * 0
            test_spec.2, // 0b0001_0000 * 0
            test_spec.2, // 0b0010_0000 * 0
            test_spec.2, // 0b0100_0000 * 0
            test_spec.2, // 0b1000_0000 * 0
        ].concat());
    }

    #[rstest]
    fn encode_byte_le_0x01(test_spec: Spec) {
        let samples = encode_byte_le(test_spec.0, 0x01).collect::<Vec<_>>();
        assert_eq!(samples.len(), 44);
        #[rustfmt::skip]
        assert_eq!(samples, [
            test_spec.2, // 0b0 start bit
            test_spec.1, // 0b0000_0001 * 1
            test_spec.2, // 0b0000_0010 * 0
            test_spec.2, // 0b0000_0100 * 0
            test_spec.2, // 0b0000_1000 * 0
            test_spec.2, // 0b0001_0000 * 0
            test_spec.2, // 0b0010_0000 * 0
            test_spec.2, // 0b0100_0000 * 0
            test_spec.2, // 0b1000_0000 * 0
            test_spec.1, // 0b1 stop bit 1
            test_spec.1, // 0b1 stop bit 2
        ].concat());
    }

    proptest! {
        #[test]
        fn encode_byte_le_always_has_the_same_length(b: u8) {
            let samples = encode_byte_le(test_spec().0, b).collect::<Vec<_>>();
            assert_eq!(samples.len(), 44);
        }

        #[test]
        fn encode_byte_le_always_has_a_start_bit(b: u8) {
            let (spec, _, zero) = test_spec();
            let samples = encode_byte_le(spec, b).collect::<Vec<_>>();
            #[rustfmt::skip]
            assert_eq!(&samples[0..4],
                zero, // 0b0 start bit
            );
        }

        #[test]
        fn encode_byte_le_always_has_two_stop_bits(b: u8) {
            let (spec, one, _) = test_spec();
            let samples = encode_byte_le(spec, b).collect::<Vec<_>>();
            #[rustfmt::skip]
            assert_eq!(&samples[36..44], [
                one, // 0b1 stop bit 1
                one, // 0b1 stop bit 2
            ].concat());
        }
    }

    proptest! {
        #[test]
        fn encode_bit_encode_same_is_always_0b1_encoded(bit in powers_of_two_u8()) {
            let (spec, one, _) = test_spec();
            let samples = encode_bit(
                spec,
                bit,
                bit,
            ).collect::<Vec<_>>();

            prop_assert_eq!(samples.len(), 4);
            prop_assert_eq!(samples, one);
        }

        #[test]
        fn encode_bit_encode_mismatching_is_always_0b0_encoded((b1, b2) in mismatching_powers_of_two_u8()) {
            let (spec, _, zero) = test_spec();
            let samples = encode_bit(
                spec,
                b1,
                b2,
            ).collect::<Vec<_>>();

            prop_assert_eq!(samples.len(), 4);
            prop_assert_eq!(samples, zero);
        }
    }

    /// A one-to-one port of the padding length calculation from py_kcs
    ///
    /// # Original in Python
    ///
    /// ```python
    /// one_pulse_len = 8 * 2 * int(FRAMERATE / FREQ / 2)
    /// padding_len = one_pulse_len * (int(FRAMERATE / one_pulse_len) * leader)
    /// ```
    const fn padding_len_pykcs(sample_rate: f32, freq: f32, leader: f32) -> f32 {
        const fn one_pulse_len_pykcs(sample_rate: f32, freq: f32) -> f32 {
            16.0 * ((sample_rate / freq) / 2.0).floor()
        }

        one_pulse_len_pykcs(sample_rate, freq)
            * ((sample_rate / one_pulse_len_pykcs(sample_rate, freq)).floor() * leader)
    }

    #[rstest]
    fn kcs_padding_len_equivalency(kcs_spec: Spec) {
        let sample_rate = kcs_spec.0.sample_rate;
        let freq = kcs_spec.0.target_freq;
        let leader = 5;

        assert_eq!(
            padding_len_pykcs(sample_rate as f32, freq as f32, leader as f32),
            padding(kcs_spec.0, leader).len() as f32,
            "padding_len_pykcs vs. padding_len"
        );
    }

    proptest! {
        #[test]
        #[ignore]
        fn padding_len_equivalency_properties((sr, tf) in sr_and_tf(), leader in 0..6_usize) {
            let spec = SquareWaveSpec {
                sample_rate: sr,
                target_freq: tf,
                ..kcs_spec().0
            };
            prop_assert_eq!(
                padding_len_pykcs(sr as f32, tf as f32, leader as f32),
                padding(spec, leader).len() as f32,
                "padding_len_pykcs vs. padding().len()"
            )
        }
    }
}
