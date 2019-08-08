use std::fmt;

/// A zero-based index that can be used as index into data structures. Entities may reuse these
/// indices.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
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

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A zero-based generation that can be used to track the number of times that a corresponding
/// index has been used previously.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Generation(u32);

impl Generation {
    #[cfg(test)]
    pub fn new(gen: u32) -> Generation {
        Generation(gen)
    }

    /// Activates the current generation. Panics if the generation is already active.
    pub fn activate(&mut self) -> Generation {
        if !self.is_active() {
            self.0 += 1;
            *self
        } else {
            panic!("Attempted to activate an active generation")
        }
    }

    /// Deactivates the current generation. Panics if the generation is already inactive.
    pub fn deactivate(&mut self) -> Generation {
        if self.is_active() {
            self.0 += 1;
            *self
        } else {
            panic!("Attempted to deactivate an inactive generation")
        }
    }

    /// Returns `true`, if the current generation is an odd number, `false` if even or zero.
    pub fn is_active(&self) -> bool {
        self.0 % 2 == 1
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
