use itertools::Itertools;
use num_traits::Signed;
use std::{borrow::Borrow, collections::VecDeque, task::Poll};

const BITMASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];

pub fn decode<N, I>(channels: usize, sample_rate: usize, target_freq: usize, samples: I) -> Vec<Vec<u8>>
where
    N: Copy + Signed,
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

    per_channel_iter
        .into_iter()
        .map(|(_, channel_samples)| decode_channel(channel_samples.map(|(_, sample)| sample), samples_per_bit))
        .collect()
}

fn decode_channel<S>(channel_data: impl Iterator<Item = S>, samples_per_bit: usize) -> Vec<u8>
where
    S: Copy + Signed,
{
    // Per-channel decoder state
    let mut look_behind: RingBuffer<u8> = RingBuffer::new(samples_per_bit);
    let mut output = Vec::default();

    let mut sign_change_iter = channel_data
        .map(|sample| to_sign_bit(sample))
        .scan(0_u8, |p, sample| Some(to_sign_change(sample, p)));

    let mut fsm = Bdec::new(sign_change_iter.by_ref(), &mut look_behind, samples_per_bit);
    while !fsm.is_complete() {
        match fsm.poll() {
            Poll::Pending => (),
            Poll::Ready(Ok(output_byte)) => output.push(output_byte),
            Poll::Ready(Err(BitDecoderError::EndOfIterator)) => break,
            Poll::Ready(Err(e)) => panic!("{e}"),
        }
    }

    output
}

#[derive(Debug)]
struct Bdec<'lt, I> {
    state: BitDecoder,
    iter: &'lt mut I,
    look_behind: &'lt mut RingBuffer<u8>,
    samples_per_bit: usize,
}

impl<'lt, I> Bdec<'lt, I>
where
    I: Iterator<Item = u8>,
{
    fn new(iter: &'lt mut I, look_behind: &'lt mut RingBuffer<u8>, samples_per_bit: usize) -> Self {
        Self {
            state: BitDecoder::default(),
            iter,
            look_behind,
            samples_per_bit,
        }
    }

    fn is_complete(&self) -> bool {
        matches!(self.state, BitDecoder::Complete)
    }

    fn poll(&mut self) -> Poll<Result<u8, BitDecoderError>> {
        let mut output = Poll::Pending;
        self.state = self
            .state
            .next(&mut self.iter, &mut self.look_behind, self.samples_per_bit, &mut output);
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BitDecoder {
    Initialize,
    DetectStartBit(usize),
    DecodeBit { mask_idx: usize, byte: u8 },
    DetectStopBits(u8),
    Complete,
}

impl Default for BitDecoder {
    fn default() -> Self {
        BitDecoder::Initialize
    }
}

#[derive(Debug, thiserror::Error)]
enum BitDecoderError {
    #[error("The sample iterator does not have anymore elements")]
    EndOfIterator,
    #[error("Could not detect one or more stop bits")]
    MissingStopBits,
}

impl BitDecoder {
    const LOW: usize = 9;
    const HIGH: usize = 12;

    fn next(
        &self,
        sign_changes: &mut impl Iterator<Item = u8>,
        look_behind: &mut RingBuffer<u8>,
        samples_per_bit: usize,
        output: &mut Poll<Result<u8, BitDecoderError>>,
    ) -> Self {
        use BitDecoder::*;
        match *self {
            Initialize => {
                look_behind.extend(sign_changes.take(samples_per_bit - 1));
                DetectStartBit(count_sign_changes(&look_behind.0))
            }
            DetectStartBit(mut num_sign_changes) => {
                let Some(current) = sign_changes.next() else {
                    *output = Poll::Ready(Err(BitDecoderError::EndOfIterator));
                    return Complete;
                };

                if current != 0 {
                    num_sign_changes += 1;
                }
                if look_behind.pop() != Some(0) {
                    num_sign_changes -= 1;
                }
                look_behind.push(current);

                if num_sign_changes <= Self::LOW {
                    DecodeBit { mask_idx: 0, byte: 0 }
                } else {
                    DetectStartBit(num_sign_changes)
                }
            }
            DecodeBit { mask_idx, mut byte } => {
                if mask_idx < BITMASKS.len() {
                    if count_sign_changes(sign_changes.take(samples_per_bit)) >= Self::HIGH {
                        byte |= BITMASKS[mask_idx];
                    }
                    DecodeBit {
                        mask_idx: mask_idx + 1,
                        byte,
                    }
                } else {
                    DetectStopBits(byte)
                }
            }
            DetectStopBits(byte) => {
                if count_sign_changes(sign_changes.take(2 * samples_per_bit)) >= Self::HIGH {
                    *output = Poll::Ready(Ok(byte));
                    Initialize
                } else {
                    *output = Poll::Ready(Err(BitDecoderError::MissingStopBits));
                    Complete
                }
            }
            Complete => Complete,
        }
    }
}

fn to_sign_bit<S: Signed + num_traits::One + std::ops::Neg>(i: S) -> u8 {
    if i.signum() == S::one().neg() { 1 } else { 0 }
}

const fn to_sign_change(i: u8, previous: &mut u8) -> u8 {
    let o = i ^ *previous;
    *previous = i;
    o
}

fn count_sign_changes<T: Borrow<u8>, I: IntoIterator<Item = T>>(i: I) -> usize {
    i.into_iter().map(|k| *k.borrow() as usize).sum()
}

#[derive(Debug, Clone, Default)]
struct RingBuffer<T>(VecDeque<T>, usize);

impl<T> RingBuffer<T> {
    fn new(size: usize) -> Self {
        RingBuffer(VecDeque::with_capacity(size + 1), size)
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
    use proptest::{prop_assert_eq, proptest};
    use rstest::rstest;

    const TEST_DIR: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests")));

    #[test]
    fn ring_buffer_initializes_with_size_plus_one() {
        let buf: RingBuffer<u8> = RingBuffer::new(1);
        assert_eq!(buf.0.capacity(), 2);
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
        assert_eq!(buf.0.len(), 1);
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn ring_buffer_push_drops_off_excess_as_fifo() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(2);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.0.len(), 2);
        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn ring_buffer_extend_drops_off_excess_as_fifo() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(2);
        buf.extend([2, 3, 4]);
        assert_eq!(buf.0.len(), 2);
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

    #[rstest]
    #[case("hello-world.wav", "hello-world.txt")]
    fn decoding_files_works_as_expected(#[case] source: &str, #[case] expected: &str) {
        use std::{
            fs::File,
            io::{BufReader, Read},
        };

        let r = WavReader::open(TEST_DIR.join(source)).unwrap();

        // Verify decoder assumptions
        let spec = r.spec();
        assert_eq!(
            spec.sample_format,
            hound::SampleFormat::Int,
            "Sample data type should be Int"
        );
        assert!(spec.bits_per_sample <= 16, "Bits per sample should be at most 16");

        let output = decode(
            spec.channels as usize,
            spec.sample_rate as usize,
            2400,
            r.into_samples::<i16>().map(|s| s.unwrap()),
        );

        let mut expected_data = Vec::new();
        BufReader::new(File::open(TEST_DIR.join(expected)).unwrap())
            .read_to_end(&mut expected_data)
            .unwrap();

        assert_eq!(output.len(), 1);
        let output = &output[0];

        //assert_eq!(output.len(), expected_data.len());
        assert_eq!(output, &expected_data);
    }
}
