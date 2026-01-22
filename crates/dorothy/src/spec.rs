use crate::util::samples_per_bit;

#[derive(Debug, Clone, PartialEq)]
pub struct Spec<S> {
    pub low: S,
    pub high: S,
    pub channels: u16,
    pub sample_rate: u32,
    pub mark_frequency: u32,
    pub space_frequency: u32,
    pub mark_num_periods: usize,
    pub space_num_periods: usize,
    pub padding_factor: usize,
    pub mark_power_threshold_db: f64,
    pub space_power_threshold_db: f64,
}

impl<S> Spec<S> {
    pub fn bit_width(&self) -> usize {
        samples_per_bit(
            self.sample_rate as usize,
            self.mark_frequency as usize,
            self.mark_num_periods,
        )
    }
}

macro_rules! impl_with_kcs {
    ($($t:ty),+) => {
        $(
            impl Spec<$t> {
                #[must_use]
                pub const fn with_kcs() -> Self {
                    Self {
                        low: -<$t>::MAX,
                        high: <$t>::MAX,
                        channels: 1,
                        sample_rate: 9600,
                        mark_frequency: 2400,
                        space_frequency: 1200,
                        mark_num_periods: 8,
                        space_num_periods: 4,
                        padding_factor: 5,
                        mark_power_threshold_db: 44.0,
                        space_power_threshold_db: -44.0,
                    }
                }
            }
        )+
    };
}

impl_with_kcs!(i8, i16, i32);
