use serde::{Serialize, Deserialize};

/// Base two log of the number of bits in a usize.
#[cfg(target_pointer_width = "64")]
const BITS: usize = 6;
#[cfg(target_pointer_width = "32")]
const BITS: usize = 5;
/// Amount of layers in the hierarchical bitset.
const LAYERS: usize = 4;
/// Maximum amount of bits per bitset.
const MAX: usize = BITS * LAYERS;
/// Highest valid index that can be stored in a bitset.
const MAX_EID: usize = 2 << MAX - 1;
/// Layer0 shift (bottom layer, true bitset).
const SHIFT0: usize = 0;
/// Layer1 shift (third layer).
const SHIFT1: usize = SHIFT0 + BITS;
/// Layer2 shift (second layer).
const SHIFT2: usize = SHIFT1 + BITS;
/// Top layer shift.
const SHIFT3: usize = SHIFT2 + BITS;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Index(u32);

impl Index {
    /// Location of the bit in the row.
    pub fn row(&self, o: usize) -> usize {
        ((self.0 >> o) as usize) & ((1 << BITS) - 1)
    }

    /// Index of the row that the bit is in.
    pub fn offset(&self, o: usize) -> usize {
        self.0 as usize / (1 << o)
    }

    /// Bitmask of the row the bit is in.
    pub fn mask(&self, o: usize) -> usize {
        1usize << self.row(o)
    }
}

impl From<u32> for Index {
    fn from(value: u32) -> Self {
        Index(value)
    }
}

impl From<Index> for u32 {
    fn from(value: Index) -> Self {
        value.0 as u32
    }
}

impl From<Index> for usize {
    fn from(value: Index) -> Self {
        value.0 as usize
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BitSet(Vec<usize>, Vec<usize>, Vec<usize>, usize);

impl BitSet {
    pub fn iter(&self) -> BitSetIter {
        BitSetIter::new(self, [0, 0, 0, self.3], [0; LAYERS - 1])
    }

    pub fn is_empty(&self) -> bool {
        self.3 == 0
    }

    pub fn contains<I: Into<Index>>(&self, idx: I) -> bool {
        let idx: Index = idx.into();
        let p0 = idx.offset(SHIFT1);

        p0 < self.0.len() && (self.0[p0] & idx.mask(SHIFT0)) != 0
    }

    /// Return `true` if the index was already in the set.
    pub fn add<I: Into<Index>>(&mut self, idx: I) -> bool {
        let idx: Index = idx.into();
        let p0 = idx.offset(SHIFT1);
        let mask = idx.mask(SHIFT0);

        if p0 >= self.0.len() {
            self.extend(idx);
        }

        if self.0[p0] & mask != 0 {
            return true;
        }

        let old = self.0[p0];
        self.0[p0] |= mask;
        if old == 0 {
            let p1 = idx.offset(SHIFT2);
            let p2 = idx.offset(SHIFT3);

            self.1[p1] |= idx.mask(SHIFT1);
            self.2[p2] |= idx.mask(SHIFT2);
            self.3 |= idx.mask(SHIFT3);
        }
        false
    }

    /// Returns `true` if the value was removed, `false` if the value was not in the set to begin
    /// with.
    pub fn remove<I: Into<Index>>(&mut self, idx: I) -> bool {
        let idx: Index = idx.into();
        let p0 = idx.offset(SHIFT1);
        let p1 = idx.offset(SHIFT2);
        let p2 = idx.offset(SHIFT3);

        if p0 >= self.0.len() {
            return false;
        }

        if self.0[p0] & idx.mask(SHIFT0) == 0 {
            return false;
        }

        self.0[p0] &= !id.mask(SHIFT0);
        if self.0[p0] != 0 {
            return true;
        }

        self.1[p1] &= !id.mask(SHIFT1);
        if self.1[p1] != 0 {
            return true;
        }

        self.2[p2] &= !id.mask(SHIFT2);
        if self.2[p2] != 0 {
            return true;
        }

        self.3 &= !id.mask(SHIFT3);
        return true;
    }

    pub fn clear(&mut self) {
        self.0.clear();
        self.1.clear();
        self.2.clear();
        self.3 = 0;
    }

    fn extend<I: Into<Index>>(&mut self, idx: I) {
        let idx: Index = idx.into();
        let p0 = idx.offset(SHIFT1);
        let p1 = idx.offset(SHIFT2);
        let p2 = idx.offset(SHIFT3);

        BitSet::assert_valid_range(idx);

        if self.0.len() <= p0 {
            self.0.resize(p0 + 1, 0);
        }
        if self.1.len() <= p1 {
            self.1.resize(p1 + 1, 0);
        }
        if self.2.len() <= p2 {
            self.2.resize(p2 + 1, 0);
        }
    }

    fn assert_valid_index<I: Into<Index>>(idx: I) {
        let idx: Index = idx.into();

        if idx > MAX_EID {
            panic!("Expected the index to be less than {}, found {}", MAX_EID, idx);
        }
    }
}

impl PartialEq for BitSet {
    fn eq(&self, rhs: &BitSet) -> bool {
        if self.3 != rhs.3 {
            return false;
        }
        if self.2.len() != rhs.2.len()
            || self.1.len() != rhs.1.len()
            || self.0.len() != rhs.0.len()
        {
            return false;
        }

        for i in 0..self.2.len() {
            if self.2(i) != rhs.2(i) {
                return false;
            }
        }
        for i in 0..self.1.len() {
            if self.1(i) != rhs.1(i) {
                return false;
            }
        }
        for i in 0..self.0.len() {
            if self.0(i) != rhs.0(i) {
                return false;
            }
        }

        true
    }
}

impl Eq for BitSet {}

pub struct BitSetIter;

impl BitSetIter {
}

impl Iterator for BitSetIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        unimplemented!()
    }
}
