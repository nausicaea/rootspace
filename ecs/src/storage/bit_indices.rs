use std::{iter::FusedIterator, mem::size_of};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BitIndices {
    inner: Vec<u32>,
}

impl BitIndices {
    pub fn with_capacity(capacity: usize) -> Self {
        BitIndices {
            inner: Vec::with_capacity((capacity - 1) / bit_size::<u32>() as usize + 1),
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity() * bit_size::<u32>() as usize
    }

    pub fn is_empty(&self) -> bool {
        if self.inner.is_empty() {
            return true;
        }

        self.inner.iter().all(|&g| g == 0)
    }

    pub fn len(&self) -> usize {
        self.inner.iter().map(|g| g.count_ones()).sum::<u32>() as usize
    }

    pub fn contains<I: Into<u32>>(&self, index: I) -> bool {
        let idx = index.into();
        let group = (idx / bit_size::<u32>()) as usize;
        let position = idx % bit_size::<u32>();

        if self.inner.len() < group + 1 {
            return false;
        }

        (self.inner[group] & (1 << position)) > 0
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

        let previously_occupied = (self.inner[group] & (1 << position)) > 0;

        self.inner[group] |= 1 << position;

        !previously_occupied
    }

    pub fn remove<I: Into<u32>>(&mut self, index: I) -> bool {
        let idx = index.into();
        let group = (idx / bit_size::<u32>()) as usize;
        let position = idx % bit_size::<u32>();

        if self.inner.len() < group + 1 {
            return false;
        }

        let previously_occupied = (self.inner[group] & (1 << position)) > 0;

        self.inner[group] &= !(1 << position);

        let remove_groups = self.inner.iter().rev().take_while(|&&g| g == 0).count();
        if remove_groups > 0 {
            self.inner.resize(self.inner.len() - remove_groups, 0);
        }

        previously_occupied
    }

    pub fn clear(&mut self) {
        self.inner.clear()
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
    num_groups: u32,
    group_cursor: u32,
    local_cursor: u32,
    entry_cursor: u32,
}

impl<'a> Iter<'a> {
    fn new(indices: &'a BitIndices) -> Self {
        Iter {
            indices,
            num_entries: indices.len() as u32,
            num_groups: indices.inner.len() as u32,
            group_cursor: 0,
            local_cursor: 0,
            entry_cursor: 0,
        }
    }

    fn increment_cursor(&mut self) -> bool {
        if self.group_cursor >= self.num_groups {
            return false;
        }

        self.entry_cursor += 1;
        self.local_cursor += 1;
        if self.local_cursor >= bit_size::<u32>() {
            self.group_cursor += 1;
            self.local_cursor = 0;
            if self.group_cursor >= self.num_groups {
                return false;
            }
        }

        true
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.group_cursor >= self.num_groups {
            return None;
        }

        while (self.indices.inner[self.group_cursor as usize] & (1 << self.local_cursor)) == 0 {
            if !self.increment_cursor() {
                return None;
            }
        }

        let item = self.group_cursor * bit_size::<u32>() + self.local_cursor;
        self.increment_cursor();
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_length = self.num_entries.saturating_sub(self.entry_cursor);
        (remaining_length as usize, Some(remaining_length as usize))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.group_cursor >= self.num_groups {
            return None;
        }

        while (self.indices.inner[(self.num_groups - self.group_cursor - 1) as usize]
            & (1 << (bit_size::<u32>() - self.local_cursor - 1)))
            == 0
        {
            if !self.increment_cursor() {
                return None;
            }
        }

        let item =
            (self.num_groups - self.group_cursor - 1) * bit_size::<u32>() + (bit_size::<u32>() - self.local_cursor - 1);
        self.increment_cursor();
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
        bi.insert(0u32);
        assert_eq!(bi.inner, vec![0b1u32]);
        bi.insert(1u32);
        assert_eq!(bi.inner, vec![0b11u32]);
        bi.insert(2u32);
        assert_eq!(bi.inner, vec![0b111u32]);
        bi.insert(45u32);
        assert_eq!(bi.inner, vec![0b111u32, 0b10000000000000u32]);
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
