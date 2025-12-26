use std::num::NonZeroU32;
use crate::encoder;

pub const fn above_nyquist_limit(frequency: u32, sampling_rate: u32) -> bool {
    (frequency + frequency) <= sampling_rate
}

pub trait CodecProperties {
    const SAMPLE_RATE: NonZeroU32;
    const ONE_FREQUENCY: NonZeroU32;
    const ZERO_FREQUENCY: NonZeroU32;
    const ONE_ENCODED: &'static [u8];
    const ZERO_ENCODED: &'static [u8];
    const HEADER_LEN: usize;
    const FOOTER_LEN: usize;

    const AMPLITUDE: u8 = 0x80;
    const CENTER: u8 = 0x80;
    const ONE_SAMPLES: usize = encoder::samples(Self::ONE_FREQUENCY, Self::SAMPLE_RATE);
    const ZERO_SAMPLES: usize = encoder::samples(Self::ZERO_FREQUENCY, Self::SAMPLE_RATE);
    const LEVEL_LOW: u8 = Self::CENTER - (Self::AMPLITUDE >> 1);
    const LEVEL_HIGH: u8 = Self::CENTER + (Self::AMPLITUDE >> 1);
    const BITMASKS: &'static [u8] = &[0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80];
}

pub struct Kcs;

impl CodecProperties for Kcs {
    const SAMPLE_RATE: NonZeroU32 = NonZeroU32::new(9600).unwrap();
    const ONE_FREQUENCY: NonZeroU32 = NonZeroU32::new(2400).unwrap();
    const ZERO_FREQUENCY: NonZeroU32 = NonZeroU32::new(1200).unwrap();
    const ONE_ENCODED: &'static [u8] = &encoder::const_square_wave::<{ Self::ONE_SAMPLES }, { Self::LEVEL_LOW }, { Self::LEVEL_HIGH }>();
    const ZERO_ENCODED: &'static [u8] = &encoder::const_square_wave::<{ Self::ZERO_SAMPLES }, { Self::LEVEL_LOW }, { Self::LEVEL_HIGH }>();

    const HEADER_LEN: usize = 5;
    const FOOTER_LEN: usize = 5;
}
