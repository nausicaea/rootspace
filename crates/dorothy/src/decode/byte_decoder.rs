use crate::ring_buffer::RingBuffer;
use crate::util::{BITMASKS, SignChange};
use std::task::Poll;

type IndexedSignChange = (usize, usize, SignChange);
type Output = Poll<Result<u8, Error>>;

#[derive(Debug)]
pub struct ByteDecoder<I> {
    state: State,
    look_behind: RingBuffer<IndexedSignChange>,
    iter: I,
    samples_per_bit: usize,
}

impl<I> ByteDecoder<I>
where
    I: Iterator<Item = IndexedSignChange>,
{
    pub fn new(iter: I, samples_per_bit: usize) -> Self {
        Self {
            state: State::default(),
            look_behind: RingBuffer::new(samples_per_bit),
            iter,
            samples_per_bit,
        }
    }

    pub const fn is_complete(&self) -> bool {
        matches!(self.state, State::Complete)
    }

    pub fn poll(&mut self) -> Output {
        let mut output = Poll::Pending;
        self.state = self.state.next(StateArgs {
            iter: self.iter.by_ref(),
            look_behind: &mut self.look_behind,
            samples_per_bit: self.samples_per_bit,
            output: &mut output,
        });
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum State {
    #[default]
    Initialize,
    DetectStartBit,
    DecodeByte {
        mask_idx: usize,
        byte: u8,
    },
    DetectStopBits(u8),
    Complete,
}

#[allow(clippy::needless_pass_by_value)]
impl State {
    const LOW: usize = 9;
    const HIGH: usize = 12;

    fn next<I: Iterator<Item = IndexedSignChange>>(self, args: StateArgs<I>) -> Self {
        match self {
            Self::Initialize => Self::initialize(args),
            Self::DetectStartBit => Self::detect_start_bit(args),
            Self::DecodeByte { mask_idx, byte } => Self::decode_byte(args, mask_idx, byte),
            Self::DetectStopBits(byte) => Self::detect_stop_bits(args, byte),
            Self::Complete => Self::Complete,
        }
    }

    fn initialize<I: Iterator<Item = IndexedSignChange>>(args: StateArgs<I>) -> Self {
        if let Err(e) = try_extend_n(args.iter, args.look_behind, args.samples_per_bit - 1) {
            *args.output = Poll::Ready(Err(e));
            Self::Complete
        } else {
            Self::DetectStartBit
        }
    }

    fn detect_start_bit<I: Iterator<Item = IndexedSignChange>>(args: StateArgs<I>) -> Self {
        let Some(current) = args.iter.next() else {
            *args.output = Poll::Ready(Err(Error::EndOfIterator));
            return Self::Complete;
        };
        args.look_behind.push_front(current);
        if args.look_behind.count_changed() <= Self::LOW {
            Self::DecodeByte { mask_idx: 0, byte: 0 }
        } else {
            Self::DetectStartBit
        }
    }

    fn decode_byte<I: Iterator<Item = IndexedSignChange>>(args: StateArgs<I>, mask_idx: usize, mut byte: u8) -> Self {
        if mask_idx < BITMASKS.len() {
            if let Err(e) = try_extend_n(args.iter, args.look_behind, args.samples_per_bit) {
                *args.output = Poll::Ready(Err(e));
                return Self::Complete;
            }
            if args.look_behind.count_changed() >= Self::HIGH {
                byte |= BITMASKS[mask_idx];
            }
            Self::DecodeByte {
                mask_idx: mask_idx + 1,
                byte,
            }
        } else {
            Self::DetectStopBits(byte)
        }
    }

    fn detect_stop_bits<I: Iterator<Item = IndexedSignChange>>(args: StateArgs<I>, byte: u8) -> Self {
        if let Err(e) = try_extend_n(args.iter, args.look_behind, 2 * args.samples_per_bit) {
            *args.output = Poll::Ready(Err(e));
            return Self::Complete;
        }
        *args.output = Poll::Ready(Ok(byte));
        Self::Initialize
        // if args.look_behind.count_changed() >= Self::HIGH {
        //     *args.output = Poll::Ready(Ok(byte));
        //     Self::Initialize
        // } else {
        //     let (sample_idx, channel_idx, _) =
        //         args.look_behind
        //             .front()
        //             .copied()
        //             .unwrap_or((0, 0, SignChange::Unchanged));
        //     *args.output = Poll::Ready(Err(Error::MissingStopBits(
        //         sample_idx - 2 * args.samples_per_bit,
        //         channel_idx,
        //     )));
        //     Self::Complete
        // }
    }
}

#[derive(Debug)]
struct StateArgs<'lt, I> {
    iter: &'lt mut I,
    look_behind: &'lt mut RingBuffer<IndexedSignChange>,
    samples_per_bit: usize,
    output: &'lt mut Output,
}

fn try_extend_n(
    i: &mut impl Iterator<Item = IndexedSignChange>,
    look_behind: &mut RingBuffer<IndexedSignChange>,
    n: usize,
) -> Result<(), Error> {
    let buf = i.take(n).collect::<Vec<_>>();
    if buf.len() == n {
        look_behind.extend(buf);
        Ok(())
    } else {
        let (sample_idx, channel_idx, _) = buf.last().copied().unwrap_or((0, 0, SignChange::Unchanged));
        Err(Error::UnexpectedEndOfIterator(sample_idx, channel_idx))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not an error: The sample iterator is complete")]
    EndOfIterator,
    #[error("Expected additional elements in the iterator after sample {0} on channel {1}")]
    UnexpectedEndOfIterator(usize, usize),
    #[error("Expected two stop bits at and after sample {0} on channel {1}")]
    MissingStopBits(usize, usize),
}
