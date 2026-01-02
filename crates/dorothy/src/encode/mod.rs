use std::iter::{repeat_n, Chain, FlatMap, RepeatN};
use std::ops::{Range, Shr};
use num_traits::{Bounded, ConstZero, Signed};
use crate::util::BITMASKS;

pub fn encode<N, I>(data: I) -> impl Iterator<Item = N>
where
    N: Copy + Signed + ConstZero + Bounded + Shr<u8, Output = N>,
    I: IntoIterator<Item = u8>,
{
    let padding_length = 5;
    padding(padding_length)
        .chain(data)
        .chain(padding(padding_length))
        .flat_map(|byte| encode_byte(byte))
}

fn encode_byte<N>(byte: u8) -> impl Iterator<Item = N>
where
    N: Copy + Signed + ConstZero + Bounded + Shr<u8, Output = N>,
{
    zero_pulse()
        .chain(
            BITMASKS.into_iter()
                .flat_map(move |mask| {
                    if byte & mask != 0 {
                        one_pulse::<N>()
                    } else {
                        zero_pulse::<N>()
                    }
                })
        )
        .chain(one_pulse())
        .chain(one_pulse())
}

fn padding(len: usize) -> impl Iterator<Item = u8> {
    repeat_n(0b1, len)
}

fn zero_pulse<N>() -> FlatMap<Range<i32>, Chain<RepeatN<N>, RepeatN<N>>, fn(i32) -> Chain<RepeatN<N>, RepeatN<N>>>
where
    N: Copy + Signed + ConstZero + Bounded + Shr<u8, Output = N>,
{
    (0..4).flat_map(|_| square_period(9600, 1200))
}

fn one_pulse<N>() -> FlatMap<Range<i32>, Chain<RepeatN<N>, RepeatN<N>>, fn(i32) -> Chain<RepeatN<N>, RepeatN<N>>>
where
    N: Copy + Signed + ConstZero + Bounded + Shr<u8, Output = N>,
{
    (0..8).flat_map(|_| square_period(9600, 2400))
}

fn square_period<N>(sample_rate: usize, target_freq: usize) -> Chain<RepeatN<N>, RepeatN<N>>
where
    N: Copy + Signed + ConstZero + Bounded + Shr<u8, Output = N>,
{
    let center = N::ZERO;
    let amplitude = N::max_value();
    let low = center - (amplitude >> 1);
    let high = center + (amplitude >> 1);
    let n = sample_rate / (target_freq << 1_u8);

    repeat_n(low, n).chain(repeat_n(high, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_period_i8() {
        let sp = square_period::<i8>(5000, 2400).collect::<Vec<_>>();
    }
}