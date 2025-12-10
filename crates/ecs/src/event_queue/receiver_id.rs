use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// A handle that allows a receiver to receive events from the related event queue.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReceiverId<E> {
    id: usize,
    #[serde(skip)]
    _e: PhantomData<E>,
}

impl<E> PartialEq<ReceiverId<E>> for ReceiverId<E> {
    fn eq(&self, other: &ReceiverId<E>) -> bool {
        self.id == other.id
    }
}

impl<E> Eq for ReceiverId<E> {}

impl<E> std::hash::Hash for ReceiverId<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<E> std::fmt::Debug for ReceiverId<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ReceiverId {{ id: {:?} }}", self.id)
    }
}

impl<E> Clone for ReceiverId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for ReceiverId<E> {}

impl<E> ReceiverId<E> {
    pub(super) fn new(id: usize) -> Self {
        ReceiverId { id, _e: PhantomData }
    }

    pub(super) fn id(self) -> usize {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{Token, assert_tokens};

    use super::*;

    #[test]
    fn receiver_id_serde() {
        let ri = ReceiverId::<()>::new(0);

        assert_tokens(&ri, &[Token::U64(0)])
    }
}
