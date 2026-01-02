use crate::ring_buffer::RingBuffer;
use crate::util::{BITMASKS, SignChange};
use std::task::Poll;

#[derive(Debug)]
pub struct BitDecoder<'lt, I> {
    state: State,
    look_behind: RingBuffer<SignChange>,
    iter: &'lt mut I,
    samples_per_bit: usize,
}

impl<'lt, I> BitDecoder<'lt, I>
where
    I: Iterator<Item = SignChange>,
{
    pub fn new(iter: &'lt mut I, samples_per_bit: usize) -> Self {
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
            .next(&mut self.iter, &mut self.look_behind, self.samples_per_bit, &mut output);
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
        &self,
        sign_changes: &mut impl Iterator<Item = SignChange>,
        look_behind: &mut RingBuffer<SignChange>,
        samples_per_bit: usize,
        output: &mut Poll<Result<u8, Error>>,
    ) -> Self {
        use State::{Complete, DecodeBit, DetectStartBit, DetectStopBits, Initialize};
        match *self {
            Initialize => {
                look_behind.extend(sign_changes.take(samples_per_bit - 1));
                DetectStartBit
            }
            DetectStartBit => {
                let Some(current) = sign_changes.next() else {
                    *output = Poll::Ready(Err(Error::EndOfIterator));
                    return Complete;
                };
                look_behind.push(current);
                if look_behind.count_changed() <= Self::LOW {
                    DecodeBit { mask_idx: 0, byte: 0 }
                } else {
                    DetectStartBit
                }
            }
            DecodeBit { mask_idx, mut byte } => {
                if mask_idx < BITMASKS.len() {
                    look_behind.extend(sign_changes.take(samples_per_bit));
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
                look_behind.extend(sign_changes.take(2 * samples_per_bit));
                if look_behind.count_changed() >= Self::HIGH {
                    *output = Poll::Ready(Ok(byte));
                    Initialize
                } else {
                    *output = Poll::Ready(Err(Error::MissingStopBits));
                    Complete
                }
            }
            Complete => Complete,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The sample iterator does not have anymore elements")]
    EndOfIterator,
    #[error("Could not detect one or more stop bits")]
    MissingStopBits,
}
