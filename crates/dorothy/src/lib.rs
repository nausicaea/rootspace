use std::{borrow::Borrow, collections::VecDeque};

pub fn decode<I: IntoIterator<Item = T>, T: Borrow<i16>>(channels: usize, sample_rate: usize, samples: I) -> Vec<u8> {
    use itertools::Itertools;

    // Fixed decoder parameters
    const BLOCK_SIZE: usize = 8196;
    const BASE_FREQ: u32 = 2400;
    let frames_per_bit: usize = (sample_rate as f64 * 8.0 / BASE_FREQ as f64).round() as usize;

    // Decoder state
    let mut bit_decoder: Vec<BitDecoder> = vec![BitDecoder::new(frames_per_bit); channels];
    let mut output = Vec::default();

    // Iterate over the input in large chunks.
    // Adjust the block size by the number of channels (since channel data is interleaved in
    // WAV)
    for chunk in &samples.into_iter().chunks(BLOCK_SIZE * channels) {
        // Per-block state (reset on every iteration)
        let mut channel_data = vec![[0_i16; BLOCK_SIZE]; channels];

        // Separate the interleaved audio channels per sample
        for (sample_idx, sample) in chunk.chunks(channels).into_iter().enumerate() {
            // Iterate over each channel in the sample
            for (channel_idx, channel) in sample.enumerate() {
                channel_data[channel_idx][sample_idx] = *channel.borrow();
            }
        }

        for (decoder, channel_data) in bit_decoder.iter_mut().zip_eq(&channel_data) {
            let sign_bits = to_sign_bits(channel_data);
            let sign_changes = to_sign_changes(&sign_bits, &mut decoder.previous_sign_bit);

            for bit_frame in sign_changes.chunks(frames_per_bit) {
                decoder.look_behind.extend(bit_frame);
                let num_sign_changes: usize = decoder.look_behind.0.iter()
                    .map(|&k| k as usize)
                    .sum();

                if let Some(decoded_byte) = decoder.state.next(num_sign_changes) {
                    output.push(decoded_byte);
                }
            }
        }
    }

    output
}

const fn to_sign_bit(i: i16) -> u8 {
    match i.signum() {
        -1 => 1,
        _ => 0,
    }
}

const fn to_sign_bits<const N: usize>(i: &[i16; N]) -> [u8; N] {
    let mut o = [0; N];
    let mut idx = 0;
    while idx < N {
        o[idx] = to_sign_bit(i[idx]);
        idx += 1;
    }
    o
}

const fn to_sign_changes<const N: usize>(i: &[u8; N], previous: &mut u8) -> [u8; N] {
    let mut o = [0; N];
    let mut idx = 0;
    while idx < N {
        o[idx] = i[idx] ^ *previous;
        *previous = i[idx];
        idx += 1;
    }
    o
}

#[derive(Debug, Clone)]
struct BitDecoder {
    state: BitDecoderFsm,
    look_behind: RingBuffer<u8>,
    previous_sign_bit: u8,
}

impl BitDecoder {
    fn new(buffer_size: usize) -> Self {
        Self { 
            state: BitDecoderFsm::default(), 
            look_behind: RingBuffer::new(buffer_size),
            previous_sign_bit: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BitDecoderFsm {
    DetectStartBit,
    DecodeDataBits {
        bitmask_idx: usize,
        current_byte: u8,
    },
    DetectStopBit {
        bit_idx: usize,
        decoded_byte: u8,
    },
}

impl BitDecoderFsm {
    const BITMASKS: [u8; 8] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80];
    const LOW_THRESHOLD: usize = 9;
    const HIGH_THRESHOLD: usize = 12;

    fn next(&mut self, num_sign_changes: usize) -> Option<u8> {
        let mut output = None;

        use BitDecoderFsm::*;
        *self = match *self {
            // Transition to decoding data if the number of sign changes is low, i.e.
            // the audio signal has a low frequency
            DetectStartBit if num_sign_changes <= Self::LOW_THRESHOLD => {
                DecodeDataBits { 
                    bitmask_idx: 0,
                    current_byte: 0x00,
                }
            },
            // Continue searching for the start bit
            DetectStartBit => DetectStartBit,
            DecodeDataBits { bitmask_idx, current_byte } => {
                if bitmask_idx < Self::BITMASKS.len() - 1 {
                    DecodeDataBits { 
                        bitmask_idx: bitmask_idx + 1,
                        current_byte: if num_sign_changes >= Self::HIGH_THRESHOLD {
                            current_byte | Self::BITMASKS[bitmask_idx]
                        } else {
                            current_byte
                        },
                    }
                } else {
                    DetectStopBit {
                        bit_idx: 0, 
                        decoded_byte: current_byte,
                    }
                }
            },
            DetectStopBit { bit_idx: 0, decoded_byte } => {
                // if num_sign_changes < Self::HIGH_THRESHOLD {
                //     eprintln!("Expected the first stop bit (high frequency)");
                // }
                DetectStopBit {
                    bit_idx: 1,
                    decoded_byte,
                }
            },
            DetectStopBit { bit_idx: 1, decoded_byte } => {
                // if num_sign_changes < Self::HIGH_THRESHOLD {
                //     eprintln!("Expected the second stop bit (high frequency)");
                // }
                output = Some(decoded_byte);
                DetectStartBit
            },
            state => panic!("Unknown state: {state:?}"),
        };

        output
    }
}

impl Default for BitDecoderFsm {
    fn default() -> Self {
        BitDecoderFsm::DetectStartBit
    }
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
        decode(2, 2, [0_i16; 4]);
        decode(2, 2, &[0_i16; 4]);
        decode(2, 2, vec![0_i16; 4]);
        decode(2, 2, &vec![0_i16; 4]);
        decode(2, 2, std::iter::once(0_i16));
    }

    #[rstest]
    fn to_sign_bit_returns_one_for_negative_numbers_and_zero_otherwise(
        #[values(-2, 0, 2)]
        input: i16,
    ) {
        assert_eq!(to_sign_bit(input), if input < 0 { 1_u8 } else { 0 });
    }

    #[rstest]
    #[case("hello-world.wav", "hello-world.txt")]
    #[case("good-example.wav", "good-example.txt")]
    fn decoding_files_works_as_expected(
        #[case] source: &str,
        #[case] expected: &str,
    ) {
        use std::{fs::File, io::{BufReader, Read}};

        let mut r = WavReader::open(TEST_DIR.join(source)).unwrap();

        // Verify decoder assumptions
        let spec = r.spec();
        assert_eq!(spec.sample_format, hound::SampleFormat::Int, "Sample data type should be Int");
        assert!(spec.bits_per_sample <= 16, "Bits per sample should be at most 16");

        let output = decode(spec.channels as usize, spec.sample_rate as usize, r.samples::<i16>().map(|s| s.unwrap()));

        let mut expected_data = Vec::new();
        BufReader::new(File::open(TEST_DIR.join(expected)).unwrap()).read_to_end(&mut expected_data).unwrap();

        //assert_eq!(output.len(), expected_data.len());
        assert_eq!(output, expected_data);
    }



}
