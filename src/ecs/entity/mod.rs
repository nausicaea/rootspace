pub mod generation;
pub mod index;

use std::cmp::Ordering;

use crate::ecs::entity::generation::Generation;
use crate::ecs::entity::index::Index;
use serde::{Deserialize, Serialize};

/// An entity serves as an identifier to an object within the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "(Index, Generation)", into = "(Index, Generation)")]
pub struct Entity {
    /// Holds the entity index.
    idx: Index,
    /// Holds the entity generation.
    gen: Generation,
}

impl Entity {
    /// Create a new entity by specifying index and generation directly.
    pub(crate) fn new<I, G>(idx: I, gen: G) -> Entity
    where
        I: Into<Index>,
        G: Into<Generation>,
    {
        Entity {
            idx: idx.into(),
            gen: gen.into(),
        }
    }

    /// Return the integer index of the entity, which can be used to index into data structures.
    pub fn idx(&self) -> Index {
        self.idx
    }

    /// Returns the integer generation of the entity, which indicates how often an entity has been reused.
    #[cfg(test)]
    pub(crate) fn gen(&self) -> Generation {
        self.gen
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.idx)
    }
}

impl PartialOrd for Entity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl Ord for Entity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl AsRef<Entity> for Entity {
    fn as_ref(&self) -> &Entity {
        &self
    }
}

impl AsRef<Index> for Entity {
    fn as_ref(&self) -> &Index {
        &self.idx
    }
}

impl AsRef<Generation> for Entity {
    fn as_ref(&self) -> &Generation {
        &self.gen
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
        let gen = parts[1].parse::<Generation>()?;

        Ok(Entity { idx, gen })
    }
}

impl From<Entity> for (Index, Generation) {
    fn from(value: Entity) -> Self {
        (value.idx, value.gen)
    }
}

impl From<(Index, Generation)> for Entity {
    fn from(value: (Index, Generation)) -> Entity {
        Entity {
            idx: value.0,
            gen: value.1,
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
        value.idx
    }
}

impl From<Entity> for Generation {
    fn from(value: Entity) -> Self {
        From::from(&value)
    }
}

impl From<&Entity> for Generation {
    fn from(value: &Entity) -> Self {
        value.gen
    }
}
