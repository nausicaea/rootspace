use crate::spec::Spec;
use crate::util::BITMASKS;
use std::borrow::Borrow;
use std::iter::FusedIterator;

/// Encode / modulate a binary data stream onto an audio signal using Kansas City Standard coding.
///
/// The implementation is loosely based on the `py-kcs` implementation by [David
/// Beazley](https://web.archive.org/web/20260111172023/https://www.dabeaz.com/py-kcs/).
pub fn encode<T, I, S>(spec: &Spec<S>, data: I) -> impl Iterator<Item = S>
where
    T: Borrow<u8>,
    I: IntoIterator<Item = T>,
    S: Copy,
{
    padding(spec)
        .chain(
            data.into_iter()
                .map(|t| *t.borrow())
                .flat_map(move |byte| encode_byte_le(spec, byte)),
        )
        .chain(padding(spec))
}

const fn padding<S: Copy>(spec: &Spec<S>) -> SquareWave<S> {
    SquareWave::with_mark(&Spec {
        mark_num_periods: spec.padding_factor * spec.mark_frequency as usize,
        ..*spec
    })
}

fn encode_byte_le<S: Copy>(spec: &Spec<S>, byte: u8) -> impl Iterator<Item = S> {
    SquareWave::with_space(spec)
        .chain(encode_byte_le_unarmored(spec, byte))
        .chain(SquareWave::with_mark(spec))
        .chain(SquareWave::with_mark(spec))
}

fn encode_byte_le_unarmored<S: Copy>(spec: &Spec<S>, byte: u8) -> impl Iterator<Item = S> {
    BITMASKS.into_iter().flat_map(move |mask| encode_bit(spec, mask, byte))
}

const fn encode_bit<S: Copy>(spec: &Spec<S>, mask: u8, byte: u8) -> SquareWave<S> {
    if byte & mask != 0 {
        SquareWave::with_mark(spec)
    } else {
        SquareWave::with_space(spec)
    }
}

#[derive(Debug, Clone)]
pub struct SquareWave<S> {
    low: S,
    high: S,
    period_length: usize,
    num_periods: usize,
    index: usize,
}

impl<S: Copy> SquareWave<S> {
    #[must_use]
    pub const fn new(low: S, high: S, period_length: usize, num_periods: usize) -> Self {
        Self {
            low,
            high,
            period_length,
            num_periods,
            index: 0,
        }
    }

    #[must_use]
    pub const fn with_mark(spec: &Spec<S>) -> Self {
        Self::new(
            spec.low,
            spec.high,
            (spec.sample_rate / spec.mark_frequency) as usize,
            spec.mark_num_periods,
        )
    }

    #[must_use]
    pub const fn with_space(spec: &Spec<S>) -> Self {
        Self::new(
            spec.low,
            spec.high,
            (spec.sample_rate / spec.space_frequency) as usize,
            spec.space_num_periods,
        )
    }

    const fn len_internal(&self) -> usize {
        self.period_length * self.num_periods
    }
}

impl<S: Copy> Iterator for SquareWave<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        let output = if self.index >= self.len_internal() {
            None
        } else if is_high(self.index, self.period_length) {
            Some(self.high)
        } else {
            Some(self.low)
        };

        self.index += 1;
        output
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.len_internal()
            .checked_sub(self.index)
            .map_or((0, Some(0)), |remaining| (remaining, Some(remaining)))
    }
}

impl<S: Copy> ExactSizeIterator for SquareWave<S> {}

impl<S: Copy> FusedIterator for SquareWave<S> {}

/// Return `true` when the square wave amplitude is `1`, and false where it is `0`.
const fn is_high(i: usize, period_length: usize) -> bool {
    !(2 * i / period_length).is_multiple_of(2)
}

#[cfg(test)]
mod tests {

    use crate::util::tests::{mismatching_powers_of_two_u8, powers_of_two_u8, sr_and_tf};

    use super::*;
    use proptest::{prop_assert_eq, proptest};
    use rstest::{fixture, rstest};

    /// Simple codec spec for easy testing
    #[fixture]
    const fn test_spec() -> &'static Spec<i8> {
        &Spec {
            channels: 1,
            padding_factor: 1,
            low: 0,
            high: i8::MAX,
            sample_rate: 4,
            mark_frequency: 2,
            space_frequency: 1,
            mark_num_periods: 2,
            space_num_periods: 1,
            mark_power_threshold_db: 45.0,
            space_power_threshold_db: -45.0,
        }
    }

    /// 0b1 encoded using [`test_spec()`]
    #[rustfmt::skip]
    #[fixture]
    const fn test_spec_one() -> &'static [i8] {
        &[0x00, 0x7F, 0x00, 0x7F]
    }

    /// 0b0 encoded using [`test_spec()`]
    #[rustfmt::skip]
    #[fixture]
    const fn test_spec_zero() -> &'static [i8] {
        &[0x00, 0x00, 0x7F, 0x7F]
    }

    const KCS_SPEC: Spec<i8> = Spec::<i8>::with_kcs();

    /// Returns a tuple with the square wave specification, and static values for 0b01 encoded and
    /// 0b00 encoded in that order.
    #[rustfmt::skip]
    #[fixture]
    const fn kcs_spec() -> &'static Spec<i8> {
        &KCS_SPEC
    }

    /// 0b1 encoded using [`kcs_spec()`]
    /// Eight periods of the high frequency tone for 0b1
    #[rustfmt::skip]
    #[fixture]
    const fn kcs_spec_one() -> &'static [i8] {
        &[
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, i8::MAX, i8::MAX,
        ]
    }

    /// 0b0 encoded using [`kcs_spec()`]
    /// Four periods of the low frequency tone for 0b0
    #[rustfmt::skip]
    #[fixture]
    const fn kcs_spec_zero() -> &'static [i8] {
        &[
            -i8::MAX, -i8::MAX, -i8::MAX, -i8::MAX, i8::MAX, i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, -i8::MAX, -i8::MAX, i8::MAX, i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, -i8::MAX, -i8::MAX, i8::MAX, i8::MAX, i8::MAX, i8::MAX,
            -i8::MAX, -i8::MAX, -i8::MAX, -i8::MAX, i8::MAX, i8::MAX, i8::MAX, i8::MAX,
        ]
    }

    #[rstest]
    #[case(2, &[false, true])]
    #[case(4, &[false, false, true, true])]
    #[case(6, &[false, false, false, true, true, true])]
    #[case(8, &[false, false, false, false, true, true, true, true])]
    fn is_high_period_pattern(#[case] p: usize, #[case] expected: &[bool]) {
        assert_eq!((0..p).map(|i| is_high(i, p)).collect::<Vec<_>>(), expected);
    }

    #[rstest]
    #[case::two_period(4, 2, &[false, true])]
    #[case::four_period(8, 4, &[false, false, true, true])]
    #[case::six_period(12, 6, &[false, false, false, true, true, true])]
    #[case::eight_period(16, 8, &[false, false, false, false, true, true, true, true])]
    fn is_high_period_pattern_repeats(#[case] max_i: usize, #[case] period: usize, #[case] expected: &[bool]) {
        use itertools::Itertools;

        let output = (0..max_i).map(|i| is_high(i, period)).chunks(period);
        for chunk in &output {
            assert_eq!(chunk.collect::<Vec<_>>(), expected);
        }
    }

    proptest! {
        #[test]
        fn is_high_2_period_is_true_for_odd_indices(i in 0..(usize::MAX / 2)) {
            prop_assert_eq!(is_high(i, 2), !i.is_multiple_of(2));
        }

        #[test]
        fn is_high_4_period_is_true_in_blocks_of_two(i in 0..(usize::MAX / 2)) {
            prop_assert_eq!(is_high(i, 4), !(i / 2).is_multiple_of(2));
        }

    }

    #[rstest]
    fn square_wave_with_amplitude_offset(test_spec: &Spec<i8>, test_spec_one: &[i8]) {
        let sqwave = SquareWave::with_mark(test_spec).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, test_spec_one);
    }

    #[test]
    fn square_wave_new_without_offset() {
        let sqwave = SquareWave::new(-i8::MAX, i8::MAX, 2, 2).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 4);
        assert_eq!(sqwave, &[-i8::MAX, i8::MAX, -i8::MAX, i8::MAX]);
    }

    #[rstest]
    fn square_wave_kcs_spec(kcs_spec: &Spec<i8>, kcs_spec_one: &[i8]) {
        let sqwave = SquareWave::with_mark(kcs_spec).collect::<Vec<_>>();
        assert_eq!(sqwave.len(), 32);
        // Exactly eight periods of a 2400 Hz tone at 9600 Hz sampling rate
        assert_eq!(sqwave, kcs_spec_one);
    }

    #[rstest]
    fn space_and_mark_are_equal_length(test_spec: &Spec<i8>, test_spec_one: &[i8], test_spec_zero: &[i8]) {
        let one = SquareWave::with_mark(&test_spec);
        assert_eq!(one.len(), 4);
        assert_eq!(one.collect::<Vec<_>>(), test_spec_one);
        let zero = SquareWave::with_space(&test_spec);
        assert_eq!(zero.len(), 4);
        assert_eq!(zero.collect::<Vec<_>>(), test_spec_zero);
    }

    #[rstest]
    fn kcs_space_and_mark_are_equal_length(kcs_spec: &Spec<i8>, kcs_spec_one: &[i8], kcs_spec_zero: &[i8]) {
        let one = SquareWave::with_mark(&kcs_spec);
        assert_eq!(one.len(), 32);
        assert_eq!(one.collect::<Vec<_>>(), kcs_spec_one);
        let zero = SquareWave::with_space(&kcs_spec);
        assert_eq!(zero.len(), 32);
        assert_eq!(zero.collect::<Vec<_>>(), kcs_spec_zero);
    }

    #[rstest]
    fn encode_byte_le_unarmored_0x01(test_spec: &Spec<i8>, test_spec_one: &[i8], test_spec_zero: &[i8]) {
        let samples = encode_byte_le_unarmored(&test_spec, 0x01).collect::<Vec<_>>();
        assert_eq!(samples.len(), 32);
        #[rustfmt::skip]
        assert_eq!(samples, [
            test_spec_one,  // 0b0000_0001 * 1
            test_spec_zero, // 0b0000_0010 * 0
            test_spec_zero, // 0b0000_0100 * 0
            test_spec_zero, // 0b0000_1000 * 0
            test_spec_zero, // 0b0001_0000 * 0
            test_spec_zero, // 0b0010_0000 * 0
            test_spec_zero, // 0b0100_0000 * 0
            test_spec_zero, // 0b1000_0000 * 0
        ].concat());
    }

    #[rstest]
    fn encode_byte_le_0x01(test_spec: &Spec<i8>, test_spec_one: &[i8], test_spec_zero: &[i8]) {
        let samples = encode_byte_le(&test_spec, 0x01).collect::<Vec<_>>();
        assert_eq!(samples.len(), 44);
        #[rustfmt::skip]
        assert_eq!(samples, [
            test_spec_zero, // 0b0 start bit
            test_spec_one,  // 0b0000_0001 * 1
            test_spec_zero, // 0b0000_0010 * 0
            test_spec_zero, // 0b0000_0100 * 0
            test_spec_zero, // 0b0000_1000 * 0
            test_spec_zero, // 0b0001_0000 * 0
            test_spec_zero, // 0b0010_0000 * 0
            test_spec_zero, // 0b0100_0000 * 0
            test_spec_zero, // 0b1000_0000 * 0
            test_spec_one,  // 0b1 stop bit 1
            test_spec_one,  // 0b1 stop bit 2
        ].concat());
    }

    proptest! {
        #[test]
        fn encode_byte_le_always_has_the_same_length(b: u8) {
            let samples = encode_byte_le(test_spec(), b).collect::<Vec<_>>();
            assert_eq!(samples.len(), 44);
        }

        #[test]
        fn encode_byte_le_always_has_a_start_bit(b: u8) {
            let samples = encode_byte_le(test_spec(), b).collect::<Vec<_>>();
            #[rustfmt::skip]
            assert_eq!(&samples[0..4],
                test_spec_zero(), // 0b0 start bit
            );
        }

        #[test]
        fn encode_byte_le_always_has_two_stop_bits(b: u8) {
            let samples = encode_byte_le(test_spec(), b).collect::<Vec<_>>();
            #[rustfmt::skip]
            assert_eq!(&samples[36..44], [
                test_spec_one(), // 0b1 stop bit 1
                test_spec_one(), // 0b1 stop bit 2
            ].concat());
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
            prop_assert_eq!(samples, test_spec_one());
        }

        #[test]
        fn encode_bit_encode_mismatching_is_always_0b0_encoded((b1, b2) in mismatching_powers_of_two_u8()) {
            let samples = encode_bit(
                test_spec(),
                b1,
                b2,
            ).collect::<Vec<_>>();

            prop_assert_eq!(samples.len(), 4);
            prop_assert_eq!(samples, test_spec_zero());
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
    fn kcs_padding_len_equivalency(kcs_spec: &Spec<i8>) {
        let sample_rate = kcs_spec.sample_rate;
        let freq = kcs_spec.mark_frequency;
        let leader = 5;

        assert_eq!(
            padding_len_pykcs(sample_rate as f32, freq as f32, leader as f32),
            padding(kcs_spec).len() as f32,
            "padding_len_pykcs vs. padding_len"
        );
    }

    proptest! {
        #[test]
        #[ignore]
        fn padding_len_equivalency_properties((sr, tf) in sr_and_tf(), leader in 0..6_usize) {
            let spec = Spec {
                sample_rate: sr,
                mark_frequency: tf,
                padding_factor: leader,
                ..*kcs_spec()
            };
            prop_assert_eq!(
                padding_len_pykcs(sr as f32, tf as f32, leader as f32),
                padding(&spec).len() as f32,
                "padding_len_pykcs vs. padding().len()"
            )
        }
    }
}
