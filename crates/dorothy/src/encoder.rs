use std::num::NonZeroU32;
use crate::shared::CodecProperties;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bit {
    Zero,
    One,
}

const fn to_bits<P: CodecProperties>(byte: u8) -> [Bit; 11] {
    debug_assert!(P::BITMASKS.len() == 8);
    let mut encoded = [Bit::Zero; 11];
    // Write the start bit
    encoded[0] = Bit::Zero;
    let mut i = 0;
    while i < 8 {
        if byte & P::BITMASKS[i] != 0 {
            encoded[i + 1] = Bit::One;
        } else {
            encoded[i + 1] = Bit::Zero;
        }
        i += 1;
    }
    // Write two stop bits
    encoded[9] = Bit::One;
    encoded[10] = Bit::One;
    encoded
}

fn sine_wave(amplitude: i16, freq: NonZeroU32, samples: usize) -> impl Iterator<Item = i16> {
    let amplitude = (amplitude as f64) / (i16::MAX as f64);
    let freq = freq.get() as f64;

    (0..samples).map(move |i| (amplitude * (std::f64::consts::TAU * freq * ((i as f64) / (samples as f64))).sin()) as i16)
}

fn encode_bit<P: CodecProperties>(bit: Bit) -> impl Iterator<Item = i16> {
    match bit {
        Bit::Zero => sine_wave(i16::MAX, P::ZERO_FREQUENCY, P::ZERO_SAMPLES),
        Bit::One => sine_wave(i16::MAX, P::ONE_FREQUENCY, P::ONE_SAMPLES),
    }
}

fn encode<P: CodecProperties>(bytes: &[u8]) -> impl Iterator<Item = i16> {
    (0..P::HEADER_LEN)
        .flat_map(|_| sine_wave(i16::MAX, P::ONE_FREQUENCY, P::ONE_SAMPLES))
        .chain(
            bytes.iter()
                .flat_map(|b| to_bits::<P>(*b))
                .flat_map(|b| encode_bit::<P>(b))
        )
        .chain((0..P::FOOTER_LEN).flat_map(|_| sine_wave(i16::MAX, P::ONE_FREQUENCY, P::ONE_SAMPLES)))
}

pub const fn const_square_wave<const M: usize, const LOW: u8, const HIGH: u8>() -> [u8; M] {
    debug_assert!(M % 2 == 0, "M must be even");
    let mut output = [0_u8; M];
    let mut i = 0;
    while i < M {
        if i < (M >> 1) {
            output[i] = LOW;
        } else {
            output[i] = HIGH;
        }
        i += 1;
    }
    output
}

pub const fn samples(f: NonZeroU32, s: NonZeroU32) -> usize {
    let f = f.get();
    let s = s.get();
    assert!(f <= (s >> 1), "Sampling rate must be greater or equal than twice the frequency");
    (s / (f << 1)) as usize
}

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, prop_compose, proptest};
    use proptest::strategy::Just;
    use super::*;

    #[test]
    fn const_square_wave_is_a_const_function() {
        const _: [u8; 4] = const_square_wave::<4, 0x00, 0xFF>();
    }

    #[test]
    fn samples_is_a_const_function() {
        const _: usize = samples(NonZeroU32::new(1).unwrap(), NonZeroU32::new(4).unwrap());
    }

    prop_compose! {
        fn freq_and_sample_rate()(s in 3..u32::MAX)(f in 1..=(s >> 1), s in Just(s)) -> (NonZeroU32, NonZeroU32) {
            (NonZeroU32::new(f).unwrap(), NonZeroU32::new(s).unwrap())
        }
    }

    // #[rustfmt::skip]
    // #[test]
    // fn encoder_read_encodes_0x00_correctly() {
    //     let mut output: Vec<u8> = Vec::new();
    //     let input: &'static [u8] = &[0x00_u8];
    //     let write_result = Encoder::new(&mut output).write(input);
    //     assert!(
    //         write_result.is_ok(),
    //         "Encoder::write() must be successful",
    //     );
    //     assert_eq!(
    //         write_result.unwrap(),
    //         output.len(),
    //         "Encoder::write() return the amount of bytes written",
    //     );
    //     assert_eq!(
    //         output,
    //         &[
    //             // Start bit
    //             0x00, 0x00, 0x80, 0x80,
    //             // Encoded 0x00
    //             0x00, 0x00, 0x80, 0x80,
    //             0x00, 0x00, 0x80, 0x80,
    //             0x00, 0x00, 0x80, 0x80,
    //             0x00, 0x00, 0x80, 0x80,
    //             0x00, 0x00, 0x80, 0x80,
    //             0x00, 0x00, 0x80, 0x80,
    //             0x00, 0x00, 0x80, 0x80,
    //             0x00, 0x00, 0x80, 0x80,
    //             // Two stop bits
    //             0x00, 0x80, 0x00, 0x80,
    //         ],
    //         "Encoder::write() must properly encode the input byte 0x00",
    //     );
    // }

    proptest! {
        #[test]
        #[should_panic(expected = "Sampling rate must be greater or equal than twice the frequency.")]
        fn samples_panics_if_frequency_gt_half_sample_rate(sample_rate in 3..u32::MAX) {
            let freq = NonZeroU32::new((sample_rate >> 1) + 1).unwrap();
            let _ = samples(freq, NonZeroU32::new(sample_rate).unwrap());
        }

        #[test]
        fn samples_calculates_the_number_of_samples_from_frequency_and_sampling_rate((f, s) in freq_and_sample_rate()) {
            let n = (s.get() as f64 / f.get() as f64 / 2.0).floor() as usize;
            prop_assert_eq!(samples(f, s), n);
        }

        #[test]
        fn sample_rate_calc_using_f64_is_equivalent_to_usize((f, s) in freq_and_sample_rate()) {
            let n_f64 = (s.get() as f64 / f.get() as f64 / 2.0).floor() as u32;
            let n_u32 = s.get() / (f.get() << 1);
            prop_assert_eq!(n_f64, n_u32);
        }

        // #[test]
        // fn encoder_encode_byte_framed_generates_a_known_number_of_bytes(input: u8) {
        //     const BITS: &'static [u8] = &[0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
        //     let ones = BITS.iter().filter(|&&bit| input & bit != 0).count();
        //     let zeros = BITS.iter().filter(|&&bit| input & bit == 0).count();
        //     let expected =
        //         // Start bit
        //         Kcs::ZERO_SAMPLES +
        //         (zeros * Kcs::ZERO_SAMPLES + ones * Kcs::ONE_SAMPLES) +
        //         // Two stop bits
        //         2 * Kcs::ONE_SAMPLES;

        //     prop_assert_eq!(
        //         encode_byte_framed(input).count(),
        //         expected,
        //         "The input byte {:x} plus three frame bits should result in {} encoded bytes at sampling rate {}",
        //         input,
        //         expected,
        //         Kcs::SAMPLE_RATE
        //     );
        // }
    }
}