use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(from = "(usize, usize, usize)", into = "(usize, usize, usize)")]
pub(crate) struct ReceiverState<E> {
    id: usize,
    pub read: usize,
    pub received: usize,
    #[serde(skip)]
    _e: PhantomData<E>,
}

impl<E> ReceiverState<E> {
    pub(crate) fn new(id: usize) -> Self {
        ReceiverState {
            id,
            read: 0,
            received: 0,
            _e: PhantomData::default(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.read = 0;
        self.received = 0;
    }
}

impl<E> PartialEq<ReceiverState<E>> for ReceiverState<E> {
    fn eq(&self, other: &ReceiverState<E>) -> bool {
        self.id == other.id && self.read == other.read && self.received == other.received
    }
}

impl<E> Clone for ReceiverState<E> {
    fn clone(&self) -> Self {
        ReceiverState {
            id: self.id,
            read: self.read,
            received: self.received,
            _e: PhantomData::default(),
        }
    }
}

impl<E> From<(usize, usize, usize)> for ReceiverState<E> {
    fn from(value: (usize, usize, usize)) -> Self {
        ReceiverState {
            id: value.0,
            read: value.1,
            received: value.2,
            _e: PhantomData::default(),
        }
    }
}

impl<E> From<ReceiverState<E>> for (usize, usize, usize) {
    fn from(value: ReceiverState<E>) -> Self {
        (value.id, value.read, value.received)
    }
}

impl<E> std::fmt::Debug for ReceiverState<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ReceiverState {{ id: {:?}, read: {:?}, received: {:?} }}",
            self.id, self.read, self.received
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn receiver_state_serde() {
        let rs = ReceiverState::<()>::new(1);

        assert_tokens(
            &rs,
            &[
                Token::Tuple { len: 3 },
                Token::U64(1),
                Token::U64(0),
                Token::U64(0),
                Token::TupleEnd,
            ],
        )
    }
}
