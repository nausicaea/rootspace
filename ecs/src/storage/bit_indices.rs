use std::{iter::FusedIterator, mem::size_of};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BitIndices {
    num_entries: usize,
    inner: Vec<u32>,
}

impl BitIndices {
    pub fn with_capacity(capacity: usize) -> Self {
        BitIndices {
            num_entries: 0,
            inner: Vec::with_capacity((capacity - 1) / bit_size::<u32>() as usize + 1),
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity() * bit_size::<u32>() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.num_entries == 0
    }

    pub fn len(&self) -> usize {
        self.num_entries
    }

    pub fn contains<I: Into<u32>>(&self, index: I) -> bool {
        let idx = index.into();
        let group = (idx / bit_size::<u32>()) as usize;
        let position = idx % bit_size::<u32>();

        if self.num_entries == 0 || self.inner.len() < group + 1 {
            return false;
        }

        (self.inner[group] & (1 << position)) != 0
    }

    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    pub fn insert<I: Into<u32>>(&mut self, index: I) -> bool {
        let idx = index.into();
        let group = (idx / bit_size::<u32>()) as usize;
        let position = idx % bit_size::<u32>();

        if group + 1 > self.inner.len() {
            self.inner.resize(group + 1, Default::default());
        }

        if (self.inner[group] & (1 << position)) != 0 {
            return false;
        }

        self.inner[group] |= 1 << position;
        self.num_entries += 1;

        true
    }

    pub fn remove<I: Into<u32>>(&mut self, index: I) -> bool {
        let idx = index.into();
        let group = (idx / bit_size::<u32>()) as usize;
        let position = idx % bit_size::<u32>();

        if self.num_entries == 0 || self.inner.len() < group + 1 {
            return false;
        }

        if (self.inner[group] & (1 << position)) == 0 {
            return false;
        }

        self.inner[group] &= !(1 << position);
        self.num_entries -= 1;

        let remove_groups = self.inner.iter().rev().take_while(|&&g| g == 0).count();
        if remove_groups > 0 {
            self.inner.resize(self.inner.len() - remove_groups, 0);
        }

        true
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.num_entries = 0;
    }
}

impl<'a> IntoIterator for &'a BitIndices {
    type IntoIter = Iter<'a>;
    type Item = u32;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

pub struct Iter<'a> {
    indices: &'a BitIndices,
    num_entries: u32,
    entry_cursor: u32,
    cursor: u32,
    rev_cursor: u32,
}

impl<'a> Iter<'a> {
    fn new(indices: &'a BitIndices) -> Self {
        let num_entries = indices.num_entries as u32;
        let num_groups = indices.inner.len() as u32;

        Iter {
            indices,
            num_entries,
            entry_cursor: 0,
            cursor: 0,
            rev_cursor: num_groups * bit_size::<u32>(),
        }
    }

    fn cursors(&self) -> (u32, u32) {
        (
            self.cursor / bit_size::<u32>(),
            self.cursor % bit_size::<u32>(),
        )
    }

    fn rev_cursors(&self) -> (u32, u32) {
        (
            (self.rev_cursor - 1) / bit_size::<u32>(),
            (self.rev_cursor - 1) % bit_size::<u32>(),
        )
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entry_cursor >= self.num_entries {
            return None;
        }
        if self.cursor >= self.indices.inner.len() as u32 * bit_size::<u32>() {
            return None;
        }

        let mut c = self.cursors();
        while (self.indices.inner[c.0 as usize] & (1 << c.1)) == 0 {
            self.cursor += 1;
            c = self.cursors();

            if self.cursor >= self.indices.inner.len() as u32 * bit_size::<u32>() {
                return None;
            }
        }

        let item = self.cursor;
        self.entry_cursor += 1;
        self.cursor += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_length = self.num_entries.saturating_sub(self.entry_cursor);
        (remaining_length as usize, Some(remaining_length as usize))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.entry_cursor >= self.num_entries {
            return None;
        }
        if self.rev_cursor == 0 {
            return None;
        }

        let mut c = self.rev_cursors();
        while self.indices.inner[c.0 as usize] & (1 << c.1) == 0 {
            self.rev_cursor -= 1;
            c = self.rev_cursors();

            if self.rev_cursor == 0 {
                return None;
            }
        }

        let item = self.rev_cursor - 1;
        self.entry_cursor += 1;
        self.rev_cursor -= 1;
        Some(item)
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> FusedIterator for Iter<'a> {}

const fn bit_size<T>() -> u32 {
    size_of::<T>() as u32 * 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitshift() {
        assert_eq!(1 << 0, 1);
        assert_eq!(1 << 1, 2);
        assert_eq!(1 << 2, 4);
    }

    #[test]
    fn bitmask() {
        assert_eq!(0b1000u32 | 0b0100u32, 0b1100u32);
        assert_ne!(0b1000u32 | 0b0100u32, 0);
        assert_eq!(0b1000u32 & 0b0100u32, 0b0000u32);
        assert_eq!(0b1000u32 & 0b0100u32, 0);
        assert_eq!(0b1100u32 & 0b0100u32, 0b0100u32);
        assert_ne!(0b1100u32 & 0b0100u32, 0);
        assert_eq!(0b1100u32 & !0b0100u32, 0b1000u32);
        assert_ne!(0b1100u32 & !0b0100u32, 0);
    }

    #[test]
    fn default() {
        let _: BitIndices = Default::default();
    }

    #[test]
    fn internal_representation() {
        let mut bi = BitIndices::default();

        assert_eq!(bi.inner, Vec::<u32>::new());
        assert_eq!(bi.num_entries, 0usize);
        bi.insert(0u32);
        assert_eq!(bi.inner, vec![0b1u32]);
        assert_eq!(bi.num_entries, 1usize);
        bi.insert(1u32);
        assert_eq!(bi.inner, vec![0b11u32]);
        assert_eq!(bi.num_entries, 2usize);
        bi.insert(2u32);
        assert_eq!(bi.inner, vec![0b111u32]);
        assert_eq!(bi.num_entries, 3usize);
        bi.insert(45u32);
        assert_eq!(bi.inner, vec![0b111u32, 0b10000000000000u32]);
        assert_eq!(bi.num_entries, 4usize);
        bi.remove(2u32);
        assert_eq!(bi.inner, vec![0b011u32, 0b10000000000000u32]);
        assert_eq!(bi.num_entries, 3usize);
        bi.remove(45u32);
        assert_eq!(bi.inner, vec![0b011u32]);
        assert_eq!(bi.num_entries, 2usize);
        bi.remove(2u32);
        assert_eq!(bi.inner, vec![0b011u32]);
        assert_eq!(bi.num_entries, 2usize);
        bi.remove(1u32);
        assert_eq!(bi.inner, vec![0b001u32]);
        assert_eq!(bi.num_entries, 1usize);
        bi.remove(0u32);
        assert_eq!(bi.inner, Vec::<u32>::new());
        assert_eq!(bi.num_entries, 0usize);
        bi.remove(0u32);
        assert_eq!(bi.inner, Vec::<u32>::new());
        assert_eq!(bi.num_entries, 0usize);
    }

    #[test]
    fn insert() {
        let mut bi = BitIndices::default();

        assert!(bi.insert(0u32));
        assert!(!bi.insert(0u32));
        assert!(bi.insert(2u32));
        assert!(!bi.insert(2u32));
        assert!(bi.insert(45u32));
        assert!(!bi.insert(45u32));
    }

    #[test]
    fn contains() {
        let mut bi = BitIndices::default();

        assert!(!bi.contains(0u32));
        bi.insert(0u32);
        assert!(bi.contains(0u32));
        assert!(!bi.contains(2u32));
        bi.insert(2u32);
        assert!(bi.contains(2u32));
        assert!(!bi.contains(45u32));
        bi.insert(45u32);
        assert!(bi.contains(2u32));
    }

    #[test]
    fn remove() {
        let mut bi = BitIndices::default();

        bi.insert(0u32);
        bi.insert(2u32);
        bi.insert(45u32);
        assert!(bi.remove(45u32));
        assert!(!bi.remove(45u32));
        assert!(bi.remove(0u32));
        assert!(!bi.remove(0u32));
        assert!(bi.remove(2u32));
        assert!(!bi.remove(2u32));
    }

    #[test]
    fn len() {
        let mut bi = BitIndices::default();

        assert_eq!(bi.len(), 0);
        bi.insert(0u32);
        assert_eq!(bi.len(), 1);
        bi.insert(2u32);
        assert_eq!(bi.len(), 2);
        bi.insert(45u32);
        assert_eq!(bi.len(), 3);
        bi.remove(0u32);
        assert_eq!(bi.len(), 2);
        bi.remove(45u32);
        assert_eq!(bi.len(), 1);
        bi.remove(2u32);
        assert_eq!(bi.len(), 0);
    }

    #[test]
    fn is_empty() {
        let mut bi = BitIndices::default();

        assert!(bi.is_empty());
        bi.insert(0u32);
        assert!(!bi.is_empty());
        bi.remove(0u32);
        assert!(bi.is_empty());
    }

    #[test]
    fn clear() {
        let mut bi = BitIndices::default();

        bi.insert(0u32);
        bi.insert(2u32);
        bi.insert(45u32);
        bi.clear();
        assert!(bi.is_empty());
    }

    #[test]
    fn with_capacity() {
        let mut bi = BitIndices::with_capacity(3);

        assert!(bi.capacity() >= 3);
        assert!(bi.is_empty());
        bi.insert(0u32);
        assert!(bi.capacity() >= 3);
        bi.insert(2u32);
        assert!(bi.capacity() >= 3);
        bi.insert(31u32);
        assert!(bi.capacity() >= 32);
        bi.insert(32u32);
        assert!(bi.capacity() >= 33);
        bi.insert(33u32);
        assert!(bi.capacity() >= 33);
        bi.insert(45u32);
        assert!(bi.capacity() >= 46);
        bi.clear();
        assert!(bi.capacity() >= 46);
    }

    #[test]
    fn impl_partial_eq() {
        assert_eq!(BitIndices::default(), BitIndices::with_capacity(45));
    }

    #[test]
    fn impl_iterator() {
        let mut bi = BitIndices::default();

        bi.insert(0u32);
        bi.insert(1u32);
        bi.insert(45u32);
        bi.insert(2u32);
        bi.insert(3u32);

        let bivec: Vec<u32> = Iter::new(&bi).collect();
        assert_eq!(bivec, [0u32, 1u32, 2u32, 3u32, 45u32]);

        let mut bit = Iter::new(&bi);
        assert_eq!(bit.size_hint(), (5, Some(5)));
        bit.next();
        assert_eq!(bit.size_hint(), (4, Some(4)));
        bit.next();
        assert_eq!(bit.size_hint(), (3, Some(3)));
        bit.next();
        assert_eq!(bit.size_hint(), (2, Some(2)));
    }

    #[test]
    fn impl_double_ended_iterator() {
        let mut bi = BitIndices::default();

        bi.insert(0u32);
        bi.insert(1u32);
        bi.insert(45u32);
        bi.insert(2u32);
        bi.insert(3u32);

        let bivec: Vec<u32> = Iter::new(&bi).rev().collect();
        assert_eq!(bivec, [45u32, 3u32, 2u32, 1u32, 0u32]);

        let mut biter = Iter::new(&bi);
        assert_eq!(biter.next(), Some(0u32));
        assert_eq!(biter.next(), Some(1u32));
        assert_eq!(biter.next(), Some(2u32));
        assert_eq!(biter.next_back(), Some(45u32));
        assert_eq!(biter.next_back(), Some(3u32));
        assert_eq!(biter.next_back(), None);
        assert_eq!(biter.next_back(), None);
        assert_eq!(biter.next_back(), None);
        assert_eq!(biter.next_back(), None);
        assert_eq!(biter.next(), None);
        assert_eq!(biter.next(), None);
        assert_eq!(biter.next(), None);
    }

    #[test]
    fn impl_exact_size_iterator() {
        let mut bi = BitIndices::default();

        bi.insert(0u32);
        bi.insert(1u32);
        bi.insert(45u32);
        bi.insert(2u32);
        bi.insert(3u32);

        assert_eq!(Iter::new(&bi).len(), 5);
    }

    #[test]
    fn impl_fused_iterator() {
        let mut bi = BitIndices::default();

        bi.insert(0u32);
        bi.insert(1u32);
        bi.insert(45u32);

        let mut biter = Iter::new(&bi);
        assert_eq!(biter.next(), Some(0u32));
        assert_eq!(biter.next(), Some(1u32));
        assert_eq!(biter.next(), Some(45u32));
        for _ in 0..(bi.capacity() + 1) {
            assert_eq!(biter.next(), None);
        }
    }

    #[test]
    fn impl_into_iterator() {
        let mut bi = BitIndices::default();

        bi.insert(0u32);
        bi.insert(1u32);
        bi.insert(45u32);
        bi.insert(2u32);
        bi.insert(3u32);

        let bivec: Vec<u32> = bi.iter().collect();
        let mut i = 0;
        for index in &bi {
            assert_eq!(index, bivec[i]);
            i += 1;
        }
    }
}
