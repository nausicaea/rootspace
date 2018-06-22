use snowflake::ProcessUniqueId;
use std::fmt::{Display, Formatter, Result as FmtResult};

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
