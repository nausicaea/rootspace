use std::{borrow::Borrow, collections::VecDeque, fmt::LowerHex};
use itertools::Itertools;
use num_traits::Signed;

const BITMASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];

pub fn decode<N, I>(channels: usize, sample_rate: usize, target_freq: usize, samples: I) -> Vec<Vec<u8>> 
where
    N: Copy + LowerHex + Signed,
    I: IntoIterator<Item = N>,
{
    // Determine how many audio samples are used to encode a single bit
    let samples_per_bit: usize = (sample_rate as f64 * 8.0 / target_freq as f64).round() as usize;

    let framed_iter = samples.into_iter()
        // Retrieve the raw sample data
        .map(|sample| *sample.borrow())
        // Separate the individual interleaved channels
        .chunks(channels);

    // Create an iterator over all audio samples, grouped by channel, indexed by time and channel
    let per_channel_iter = framed_iter.into_iter()
        // Create an index for each channel
        .flat_map(|frame| frame.enumerate())
        // Group by channels, thus de-interleaving audio samples for each channel
        .chunk_by(|(channel_idx, _)| *channel_idx);

    per_channel_iter.into_iter()
        .map(|(_, channel_samples)| {
            decode_channel(
                channel_samples.map(|(_, sample)| sample), 
                samples_per_bit,
            )
        })
        .collect()
}

fn to_sign_bit<S: Signed + num_traits::One + std::ops::Neg>(i: S) -> u8 {
    if i.signum() == S::one().neg() {
        1
    } else {
        0
    }
}

const fn to_sign_change(i: u8, previous: &mut u8) -> u8 {
    let o = i ^ *previous;
    *previous = i;
    o
}

fn advance_n<I: Iterator<Item = u8>>(buf: &mut RingBuffer<u8>, i: &mut I, n: usize) {
    i
        .take(n)
        .for_each(|item| buf.push(item));
}

fn count_sign_changes<T: Borrow<u8>, I: IntoIterator<Item = T>>(i: I) -> usize {
    i.into_iter().map(|k| *k.borrow() as usize).sum()
}


fn decode_channel<S>(
    channel_data: impl Iterator<Item = S>, 
    samples_per_bit: usize,
) -> Vec<u8> 
where
    S: Copy + LowerHex + Signed,
{
    // Per-channel decoder state
    let mut look_behind: RingBuffer<u8> = RingBuffer::new(samples_per_bit);
    let mut previous_sign_bit = 0_u8;
    let mut output = Vec::default();

    let mut sign_change_iter = channel_data
        .inspect(|sample| eprint!("{sample:x} -> "))
        .map(|sample| to_sign_bit(sample))
        .inspect(|sample| eprint!("{sample:x} -> "))
        .map(|sample| to_sign_change(sample, &mut previous_sign_bit))
        .inspect(|sample| eprintln!("{sample:x}"));

    advance_n(&mut look_behind, sign_change_iter.by_ref(), samples_per_bit - 1);
    let mut num_sign_changes = count_sign_changes(&look_behind.0);

    while let Some(sign_change) = sign_change_iter.next() {
        if sign_change != 0 {
            num_sign_changes += 1;
        }
        if look_behind.pop() != Some(0) {
            num_sign_changes -= 1;
        }
        look_behind.push(sign_change);

        // If a start bit is detected, sample the next 8 data bits
        if num_sign_changes <= 9 {
            let mut byteval = 0_u8;
            for mask in BITMASKS {
                advance_n(&mut look_behind, sign_change_iter.by_ref(), samples_per_bit);
                if count_sign_changes(&look_behind.0) >= 12 {
                    byteval |= mask;
                }
            }
            output.push(byteval);

            // Skip the final two stop bits and refill the sample buffer
            sign_change_iter.by_ref()
                .skip(2 * samples_per_bit)
                .take(3 * samples_per_bit - 1)
                .for_each(|item| look_behind.push(item));
            num_sign_changes = count_sign_changes(&look_behind.0);
        }
    }

    output
}

#[derive(Debug, Clone, Default)]
struct RingBuffer<T>(VecDeque<T>, usize);

impl<T> RingBuffer<T> {
    fn new(size: usize) -> Self {
        RingBuffer(VecDeque::with_capacity(size + 1), size)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, value: T) {
        self.0.push_front(value);
        self.truncate();

    }

    fn pop(&mut self) -> Option<T> {
        self.0.pop_back()
    }

    fn truncate(&mut self) {
        if self.0.len() > self.1 {
            self.0.truncate(self.1);
        }
    }
}

impl<T: Clone, U: Borrow<T>> Extend<U> for RingBuffer<T> {
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        for element in iter {
            self.push(element.borrow().clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::LazyLock};

    use super::*;
    use hound::WavReader;
    use rstest::rstest;
    use proptest::{proptest, prop_assert_eq};

    const TEST_DIR: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests")));

    #[test]
    fn ring_buffer_initializes_with_size_plus_one() {
        let buf: RingBuffer<u8> = RingBuffer::new(1);
        assert_eq!(buf.0.capacity(), 2);
    }

    #[test]
    fn ring_buffer_len_is_never_larger_than_size() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(3);
        assert_eq!(buf.len(), 0, "0/3");
        buf.push(1);
        assert_eq!(buf.len(), 1, "1/3");
        buf.push(2);
        assert_eq!(buf.len(), 2, "2/3");
        buf.push(3);
        assert_eq!(buf.len(), 3, "3/3");
        buf.push(4);
        assert_eq!(buf.len(), 3, "4/3 -> 3/3");
    }

    #[test]
    fn ring_buffer_truncate_keeps_the_deque_length_constant() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(2);
        assert_eq!(buf.0, &[]);
        buf.truncate();
        assert_eq!(buf.0, &[]);
        buf.push(1);
        assert_eq!(buf.0, &[1]);
        buf.truncate();
        assert_eq!(buf.0, &[1]);
        buf.push(1);
        buf.push(2);
        assert_eq!(buf.0, &[2, 1]);
        buf.truncate();
        assert_eq!(buf.0, &[2, 1]);

    }

    #[test]
    fn ring_buffer_push_single_element() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(1);
        buf.push(1);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn ring_buffer_push_drops_off_excess_as_fifo() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(2);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn ring_buffer_extend_drops_off_excess_as_fifo() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(2);
        buf.extend([2, 3, 4]);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn decode_accepts_i16_iterator_input() {
        decode(1, 4, 2400, [0_i16; 4]);
        decode(1, 4, 2400, vec![0_i16; 4]);
        decode(1, 4, 2400, std::iter::once(0_i16));
    }

    #[rstest]
    fn to_sign_bit_returns_one_for_negative_numbers_and_zero_otherwise(
        #[values(-2, 0, 2)]
        input: i16,
    ) {
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

    #[rstest]
    #[case("hello-world.wav", "hello-world.txt")]
    //#[case("good-example.wav", "good-example.txt")]
    fn decoding_files_works_as_expected(
        #[case] source: &str,
        #[case] expected: &str,
    ) {
        use std::{fs::File, io::{BufReader, Read}};

        let r = WavReader::open(TEST_DIR.join(source)).unwrap();

        // Verify decoder assumptions
        let spec = r.spec();
        assert_eq!(spec.sample_format, hound::SampleFormat::Int, "Sample data type should be Int");
        assert!(spec.bits_per_sample <= 16, "Bits per sample should be at most 16");

        let output = decode(spec.channels as usize, spec.sample_rate as usize, 2400, r.into_samples::<i16>().map(|s| s.unwrap()));

        let mut expected_data = Vec::new();
        BufReader::new(File::open(TEST_DIR.join(expected)).unwrap()).read_to_end(&mut expected_data).unwrap();

        assert_eq!(output.len(), 1);
        let output = &output[0];

        //assert_eq!(output.len(), expected_data.len());
        assert_eq!(output, &expected_data);
    }



}
