use serde::{Deserialize, Serialize};

/// A zero-based index that can be used as index into data structures. Entities may reuse these
/// indices.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Index(u32);

impl Index {
    /// Create a new index.
    pub fn new(idx: u32) -> Index {
        Index(idx)
    }

    pub fn idx(&self) -> u32 {
        self.0
    }

    /// Return a copy of the current index and then increments the current index.
    pub fn post_increment(&mut self) -> Index {
        let tmp = *self;
        self.0 += 1;
        tmp
    }
}

impl PartialEq<u32> for Index {
    fn eq(&self, other: &u32) -> bool {
        &self.0 == other
    }
}

impl PartialEq<Index> for u32 {
    fn eq(&self, other: &Index) -> bool {
        self == &other.0
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Index> for Index {
    fn as_ref(&self) -> &Index {
        &self
    }
}

impl AsRef<u32> for Index {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl std::str::FromStr for Index {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx: u32 = s.parse()?;
        Ok(Index(idx))
    }
}

impl From<u32> for Index {
    fn from(value: u32) -> Self {
        Index(value)
    }
}

impl From<&u32> for Index {
    fn from(value: &u32) -> Self {
        Index(*value)
    }
}

impl From<usize> for Index {
    fn from(value: usize) -> Self {
        Index(value as u32)
    }
}

impl From<&usize> for Index {
    fn from(value: &usize) -> Self {
        Index(*value as u32)
    }
}

impl From<Index> for u32 {
    fn from(value: Index) -> Self {
        value.0
    }
}

impl From<&Index> for u32 {
    fn from(value: &Index) -> Self {
        value.0
    }
}

impl From<Index> for usize {
    fn from(value: Index) -> Self {
        value.0 as usize
    }
}

impl From<&Index> for usize {
    fn from(value: &Index) -> Self {
        value.0 as usize
    }
}
