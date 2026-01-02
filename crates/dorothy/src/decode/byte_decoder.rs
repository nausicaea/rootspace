use crate::ring_buffer::RingBuffer;
use crate::util::{BITMASKS, SignChange};
use std::task::Poll;

type IndexedSignChange = (usize, usize, SignChange);

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

    pub fn poll(&mut self) -> Poll<Result<u8, Error>> {
        let mut output = Poll::Pending;
        self.state = self
            .state
            .next(self.iter.by_ref(), &mut self.look_behind, self.samples_per_bit, &mut output);
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum State {
    #[default]
    Initialize,
    DetectStartBit,
    DecodeBit {
        mask_idx: usize,
        byte: u8,
    },
    DetectStopBits(u8),
    Complete,
}

impl State {
    const LOW: usize = 9;
    const HIGH: usize = 12;

    fn next(
        self,
        sign_changes: &mut impl Iterator<Item = IndexedSignChange>,
        look_behind: &mut RingBuffer<IndexedSignChange>,
        samples_per_bit: usize,
        output: &mut Poll<Result<u8, Error>>,
    ) -> Self {
        use State::{Complete, DecodeBit, DetectStartBit, DetectStopBits, Initialize};
        match self {
            Initialize => {
                if let Err(e) = try_extend_n(sign_changes, look_behind, samples_per_bit - 1) {
                    *output = Poll::Ready(Err(e));
                    return Complete;
                }
                DetectStartBit
            }
            DetectStartBit => {
                let Some(current) = sign_changes.next() else {
                    *output = Poll::Ready(Err(Error::EndOfIterator));
                    return Complete;
                };
                look_behind.push_front(current);
                if look_behind.count_changed() <= Self::LOW {
                    DecodeBit { mask_idx: 0, byte: 0 }
                } else {
                    DetectStartBit
                }
            }
            DecodeBit { mask_idx, mut byte } => {
                if mask_idx < BITMASKS.len() {
                    if let Err(e) = try_extend_n(sign_changes, look_behind, samples_per_bit) {
                        *output = Poll::Ready(Err(e));
                        return Complete;
                    }
                    if look_behind.count_changed() >= Self::HIGH {
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
                if let Err(e) = try_extend_n(sign_changes, look_behind, 2 * samples_per_bit) {
                    *output = Poll::Ready(Err(e));
                    return Complete;
                }
                if look_behind.count_changed() >= Self::HIGH {
                    *output = Poll::Ready(Ok(byte));
                    Initialize
                } else {
                    let (sample_idx, channel_idx, _) = look_behind.front().copied().unwrap_or((0, 0, SignChange::Unchanged));
                    *output = Poll::Ready(Err(Error::MissingStopBits(sample_idx - 2 * samples_per_bit, channel_idx)));
                    Complete
                }
            }
            Complete => Complete,
        }
    }
}

fn try_extend_n(i: &mut impl Iterator<Item = IndexedSignChange>, look_behind: &mut RingBuffer<IndexedSignChange>, n: usize) -> Result<(), Error> {
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
