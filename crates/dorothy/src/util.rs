use num_traits::{Signed, Zero};

pub fn to_sign_bit<S: Copy + Signed + Zero + PartialOrd>(i: S) -> Sign {
    if i.signum() < S::zero() {
        Sign::Negative
    } else {
        Sign::NonNegative
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Negative,
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
pub fn samples_per_bit(sample_rate: usize, target_freq: usize) -> usize {
    (sample_rate as f32 * 8.0 / target_freq as f32).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::{Arbitrary, Just, Strategy};
    use proptest::{prop_assert_eq, prop_compose, prop_oneof, proptest};
    use rstest::rstest;

    fn sign() -> impl Strategy<Value = Sign> {
        prop_oneof! {
            1 => Just(Sign::Negative),
            1 => Just(Sign::NonNegative),
        }
    }

    #[rstest]
    fn to_sign_bit_returns_one_for_negative_numbers_and_zero_otherwise(#[values(-2, 0, 2)] input: i16) {
        assert_eq!(
            to_sign_bit(input),
            if input < 0 { Sign::Negative } else { Sign::NonNegative }
        );
    }

    proptest! {
        #[test]
        fn to_sign_bit_is_one_for_negative_input(input: i16) {
            prop_assert_eq!(to_sign_bit(input), if input < 0 { Sign::Negative } else { Sign::NonNegative });
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
}

pub const BITMASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
