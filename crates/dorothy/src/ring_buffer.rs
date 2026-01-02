use crate::util::SignChange;
use std::borrow::Borrow;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct RingBuffer<T>(VecDeque<T>, usize);

impl<T> RingBuffer<T> {
    pub fn new(size: usize) -> Self {
        Self(VecDeque::with_capacity(size + 1), size)
    }

    pub fn push_front(&mut self, value: T) {
        self.0.push_front(value);
        self.truncate();
    }
    
    pub fn front(&self) -> Option<&T> {
        self.0.front()
    }

    fn truncate(&mut self) {
        if self.0.len() > self.1 {
            self.0.truncate(self.1);
        }
    }
}

impl RingBuffer<(usize, usize, SignChange)> {
    pub fn count_changed(&self) -> usize {
        self.0.iter().filter(|(_, _, item)| matches!(item, SignChange::Changed)).count()
    }
}

impl<T: Copy, U: Borrow<T>> Extend<U> for RingBuffer<T> {
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        for element in iter {
            self.push_front(*element.borrow());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        buf.push_front(1);
        assert_eq!(buf.0, &[1]);
        buf.truncate();
        assert_eq!(buf.0, &[1]);
        buf.push_front(1);
        buf.push_front(2);
        assert_eq!(buf.0, &[2, 1]);
        buf.truncate();
        assert_eq!(buf.0, &[2, 1]);
    }

    #[test]
    fn ring_buffer_push_single_element() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(1);
        buf.push_front(1);
        assert_eq!(buf.0.len(), 1);
        assert_eq!(buf.0.pop_back(), Some(1));
        assert_eq!(buf.0.pop_back(), None);
    }

    #[test]
    fn ring_buffer_push_drops_off_excess_as_fifo() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(2);
        buf.push_front(1);
        buf.push_front(2);
        buf.push_front(3);
        assert_eq!(buf.0.len(), 2);
        assert_eq!(buf.0.pop_back(), Some(2));
        assert_eq!(buf.0.pop_back(), Some(3));
        assert_eq!(buf.0.pop_back(), None);
    }

    #[test]
    fn ring_buffer_extend_drops_off_excess_as_fifo() {
        let mut buf: RingBuffer<u8> = RingBuffer::new(2);
        buf.extend([2, 3, 4]);
        assert_eq!(buf.0.len(), 2);
        assert_eq!(buf.0.pop_back(), Some(3));
        assert_eq!(buf.0.pop_back(), Some(4));
        assert_eq!(buf.0.pop_back(), None);
    }
}
