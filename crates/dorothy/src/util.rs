use num_traits::{One, Signed};
use std::ops::Neg;

pub fn to_sign_bit<S: Copy + Signed + One + Neg>(i: S) -> u8 {
    u8::from(i.signum() == S::one().neg())
}

pub const fn to_sign_change(i: u8, previous: &mut u8) -> u8 {
    let o = i ^ *previous;
    *previous = i;
    o
}

/// Determine how many audio samples are used to encode a single bit
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
pub fn samples_per_bit(sample_rate: usize, target_freq: usize) -> usize {
    (sample_rate as f32 * 8.0 / target_freq as f32).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prop_assert_eq, proptest};
    use rstest::rstest;

    #[rstest]
    fn to_sign_bit_returns_one_for_negative_numbers_and_zero_otherwise(#[values(-2, 0, 2)] input: i16) {
        assert_eq!(to_sign_bit(input), if input < 0 { 1_u8 } else { 0 });
    }

    proptest! {
        #[test]
        fn to_sign_bit_is_one_for_negative_input(input: i16) {
            prop_assert_eq!(to_sign_bit(input), if input < 0 { 1_u8 } else { 0 });
        }

        #[test]
        fn to_sign_change_xors_input_and_previous(input: u8, previous: u8) {
            let mut p = previous;
            prop_assert_eq!(to_sign_change(input, &mut p), input ^ previous);
            prop_assert_eq!(p, input);
        }
    }
}

pub const BITMASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
