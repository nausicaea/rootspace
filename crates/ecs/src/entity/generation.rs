use serde::{Deserialize, Serialize};

/// A zero-based generation that can be used to track the number of times that a corresponding
/// index has been used previously.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[must_use]
pub struct Generation(u32);

impl Generation {
    #[cfg(test)]
    pub fn new(r#gen: u32) -> Generation {
        Generation(r#gen)
    }

    /// Activates the current generation. Panics if the generation is already active.
    pub fn activate(&mut self) {
        if self.is_active() {
            panic!("Attempted to activate an active generation")
        } else {
            self.0 += 1;
        }
    }

    /// Deactivates the current generation. Panics if the generation is already inactive.
    pub fn deactivate(&mut self) {
        if self.is_active() {
            self.0 += 1;
        } else {
            panic!("Attempted to deactivate an inactive generation")
        }
    }

    /// Returns `true`, if the current generation is an odd number, `false` if even or zero.
    #[must_use] 
    pub fn is_active(&self) -> bool {
        self.0 % 2 == 1
    }
}

impl PartialEq<u32> for Generation {
    fn eq(&self, other: &u32) -> bool {
        &self.0 == other
    }
}

impl PartialEq<Generation> for u32 {
    fn eq(&self, other: &Generation) -> bool {
        self == &other.0
    }
}

impl std::fmt::Display for Generation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<u32> for Generation {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl std::str::FromStr for Generation {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r#gen: u32 = s.parse()?;
        Ok(Generation(r#gen))
    }
}

impl From<u32> for Generation {
    fn from(value: u32) -> Self {
        Generation(value)
    }
}

impl From<&u32> for Generation {
    fn from(value: &u32) -> Self {
        Generation(*value)
    }
}

impl TryFrom<usize> for Generation {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(Generation)
    }
}

impl TryFrom<&usize> for Generation {
    type Error = std::num::TryFromIntError;

    fn try_from(value: &usize) -> Result<Self, Self::Error> {
        u32::try_from(*value).map(Generation)
    }
}

impl From<Generation> for u32 {
    fn from(value: Generation) -> Self {
        value.0
    }
}

impl From<&Generation> for u32 {
    fn from(value: &Generation) -> Self {
        value.0
    }
}

impl From<Generation> for usize {
    fn from(value: Generation) -> Self {
        value.0 as usize
    }
}

impl From<&Generation> for usize {
    fn from(value: &Generation) -> Self {
        value.0 as usize
    }
}
