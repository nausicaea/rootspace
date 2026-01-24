use num_traits::Signed;
use numenor::ConstZero;

/// These values are used to split a byte into individual bits during encoding / decoding
pub const BITMASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];

/// Determine if the input is negative or non-negative. `Sign` is an artificial
/// bound, but the function just doesn't make sense otherwise,
pub fn to_sign<S: Copy + Signed + ConstZero + PartialOrd>(i: S) -> Sign {
    if i >= S::ZERO {
        Sign::NonNegative
    } else {
        Sign::Negative
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// Represents a negative number
    Negative,
    /// Represents a positive number or zero
    NonNegative,
}

pub const fn to_sign_change(i: Sign, previous: &mut Sign) -> SignChange {
    let o = match (i, *previous) {
        (Sign::Negative, Sign::NonNegative) | (Sign::NonNegative, Sign::Negative) => SignChange::Changed,
        _ => SignChange::Unchanged,
    };
    *previous = i;
    o
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignChange {
    Changed,
    Unchanged,
}

/// Determine how many audio samples are used to encode a single bit
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
pub const fn samples_per_bit(sample_rate: usize, target_freq: usize, num_periods: usize) -> usize {
    (num_periods * sample_rate) / target_freq
}

/// Convert a bit to a non-return-to-zero (NRZ) value: Bits are represented either as `+1.0` for `0b1` or `-1.0` for `0b0`.
pub const fn to_nrz(bit: bool) -> f64 {
    (bit as i8 * 2 - 1) as f64
}

#[allow(dead_code)]
pub const fn to_le_bits(byte: u8) -> [bool; 8] {
    const BITMASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
    let mut output = [false; 8];
    let mut i = 0;
    while i < 8 {
        output[i] = (byte & BITMASKS[i]) != 0;
        i += 1;
    }
    output
}

/// Calculate the power ratio on a logarithmic scale (as decibel units)
pub fn to_decibel(p: f64, p_0: f64) -> f64 {
    10.0 * ((p + 1e-10) / (p_0 + 1e-10)).log10()
}

#[cfg(test)]
pub(crate) mod tests {
    use std::ops::Range;

    use super::*;
    use proptest::prelude::{Just, Strategy};
    use proptest::sample::select;
    use proptest::{prop_assert_eq, prop_compose, prop_oneof, proptest};
    use rstest::rstest;

    fn sign() -> impl Strategy<Value = Sign> {
        prop_oneof! {
            1 => Just(Sign::Negative),
            1 => Just(Sign::NonNegative),
        }
    }

    prop_compose! {
        pub fn sr_and_tf()(sr in 2_u32..(u32::MAX >> 3))(sr in Just(sr), tf in 1..=(sr >> 1)) -> (u32, u32) {
            (sr, tf)
        }
    }

    prop_compose! {
        pub fn odd_usize()(i in 1..(usize::MAX / 2)) -> usize {
            i * 2 - 1
        }
    }

    prop_compose! {
        pub fn powers_of_two_u8()(p in 0_u8..7) -> u8 {
            2 << p
        }
    }

    fn mismatching(src: Range<u8>, el: u8) -> Vec<u8> {
        src.filter(move |&e| e != el).collect()
    }

    prop_compose! {
        pub fn mismatching_powers_of_two_u8()(p1 in 0_u8..7)(p1 in Just(p1), p2 in select(mismatching(0_u8..7, p1))) -> (u8, u8) {
            (2 << p1, 2 << p2)
        }
    }

    proptest! {
        #[test]
        fn samples_per_bit_f32_and_usize_are_equivalent((sr, tf) in sr_and_tf()) {
            let spb_f32 = (sr as f32 * 8.0 / tf as f32).floor() as usize;
            let spb_usize = samples_per_bit(sr as usize, tf as usize, 8);
            assert_eq!(spb_f32, spb_usize);
        }
    }

    #[rstest]
    fn to_sign_detects_sign_special(#[values(-2, 0, 2)] input: i16) {
        assert_eq!(
            to_sign(input),
            if input < 0 { Sign::Negative } else { Sign::NonNegative }
        );
    }

    proptest! {
        #[test]
        fn to_sign_detects_sign(input: i16) {
            prop_assert_eq!(to_sign(input), if input < 0 { Sign::Negative } else { Sign::NonNegative });
        }

        #[test]
        fn to_sign_change_xors_input_and_previous(input in sign(), previous in sign()) {
            let mut p = previous;
            let expected = match (input, previous) {
                (Sign::Negative, Sign::NonNegative) | (Sign::NonNegative, Sign::Negative) => SignChange::Changed,
                _ => SignChange::Unchanged,
            };
            prop_assert_eq!(to_sign_change(input, &mut p), expected);
            prop_assert_eq!(p, input);
        }
    }

    #[rstest]
    #[case::true_is_plus_one(true, 1.0)]
    #[case::false_is_minus_one(false, -1.0)]
    fn to_nrz_output(#[case] input: bool, #[case] output: f64) {
        assert_eq!(to_nrz(input), output);
    }

    #[rstest]
    #[case::xff_all_ones(0xff, &[true; 8])]
    #[case::x00_all_zeros(0x00, &[false; 8])]
    fn to_le_bits_output(#[case] input: u8, #[case] output: &[bool]) {
        let bits = to_le_bits(input);
        assert_eq!(bits, output);
    }
}
