pub mod generation;
pub mod index;

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::entity::{generation::Generation, index::Index};

/// An entity serves as an identifier to an object within the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "(Index, Generation)", into = "(Index, Generation)")]
pub struct Entity {
    /// Holds the entity index.
    index: Index,
    /// Holds the entity generation.
    generation: Generation,
}

impl Entity {
    /// Create a new entity by specifying index and generation directly.
    pub fn new<I: Into<Index>, G: Into<Generation>>(idx: I, generation: G) -> Entity {
        Entity {
            index: idx.into(),
            generation: generation.into(),
        }
    }

    /// Return the integer index of the entity, which can be used to index into data structures.
    pub fn idx(&self) -> Index {
        self.index
    }

    /// Returns the integer generation of the entity, which indicates how often an entity has been reused.
    #[cfg(test)]
    pub fn r#gen(&self) -> Generation {
        self.generation
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl PartialOrd for Entity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl AsRef<Entity> for Entity {
    fn as_ref(&self) -> &Entity {
        self
    }
}

impl AsRef<Index> for Entity {
    fn as_ref(&self) -> &Index {
        &self.index
    }
}

impl AsRef<Generation> for Entity {
    fn as_ref(&self) -> &Generation {
        &self.generation
    }
}

impl std::str::FromStr for Entity {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s
            .trim_matches(|p| p == '(' || p == ')' || p == ' ')
            .split(',')
            .collect();

        let idx = parts[0].parse::<Index>()?;
        let r#gen = parts[1].parse::<Generation>()?;

        Ok(Entity { index: idx, generation: r#gen })
    }
}

impl From<Entity> for (Index, Generation) {
    fn from(value: Entity) -> Self {
        (value.index, value.generation)
    }
}

impl From<(Index, Generation)> for Entity {
    fn from(value: (Index, Generation)) -> Entity {
        Entity {
            index: value.0,
            generation: value.1,
        }
    }
}

impl From<Entity> for Index {
    fn from(value: Entity) -> Self {
        From::from(&value)
    }
}

impl From<&Entity> for Index {
    fn from(value: &Entity) -> Self {
        value.index
    }
}

impl From<Entity> for Generation {
    fn from(value: Entity) -> Self {
        From::from(&value)
    }
}

impl From<&Entity> for Generation {
    fn from(value: &Entity) -> Self {
        value.generation
    }
}
