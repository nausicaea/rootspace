//! The `Entity` serves as a unique identifier for an object in the world.

use snowflake::ProcessUniqueId;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The `Entity` serves as a unique identifier for an object in the world. To achieve that, it
/// makes use of the `snowflake` crate, which can generate per-process unique numbers.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entity(ProcessUniqueId);

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Entity({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    quickcheck! {
        fn entities_are_unique(num_entities: usize) -> bool {
            let mut entities = HashSet::new();
            for _ in 0..num_entities {
                entities.insert(Entity::default());
            }

            entities.len() == num_entities
        }
    }
}
