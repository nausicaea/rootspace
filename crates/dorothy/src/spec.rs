#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spec<S> {
    pub low: S,
    pub high: S,
    pub channels: u16,
    pub sample_rate: u32,
    pub frequency: u32,
    pub num_periods: usize,
    pub padding_factor: usize,
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
                        frequency: 2400,
                        num_periods: 8,
                        padding_factor: 5,
                    }
                }
            }
        )+
    };
}

impl_with_kcs!(i8, i16, i32);
