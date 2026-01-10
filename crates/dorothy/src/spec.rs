use crate::encode::SquareWaveSpec;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spec<S> {
    pub amplitude: S,
    pub offset: S,
    pub channels: u16,
    pub sample_rate: u32,
    pub frequency: u32,
    pub num_periods: usize,
    pub padding_factor: usize,
}

impl<N: numenor::ConstBounded + numenor::ConstZero> Spec<N> {
    #[must_use]
    pub const fn with_kcs() -> Self {
        Self {
            amplitude: N::MAX,
            offset: N::ZERO,
            channels: 1,
            sample_rate: 9600,
            frequency: 2400,
            num_periods: 8,
            padding_factor: 5,
        }
    }
}

impl Spec<i8> {
    pub(crate) const fn to_sqw_spec(&self) -> SquareWaveSpec {
        SquareWaveSpec {
            offset: self.offset,
            amplitude: self.amplitude,
            sample_rate: self.sample_rate as usize,
            target_freq: self.frequency as usize,
            num_periods: self.num_periods,
        }
    }
}
