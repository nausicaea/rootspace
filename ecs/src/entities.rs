//! Provides facilities for reasoning about entities (e.g. objects) within a world.

use crate::{
    entity::{Entity, generation::Generation, index::Index},
    resource::Resource,
    SerializationProxy,
};
use serde::{Deserialize, Serialize};

/// The `Entities` resource keeps track of all entities.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Entities {
    /// Stores the highest assigned `Entity` index plus one.
    max_idx: Index,
    /// Stores all previously assigned `Entity` indices that are now available again.
    free_idx: Vec<Index>,
    /// Stores the generations of each `Entity`.
    generations: Vec<Generation>,
}

impl Entities {
    /// Create a new `Entity`.
    pub fn create(&mut self) -> Entity {
        let idx = if let Some(idx) = self.free_idx.pop() {
            idx
        } else {
            self.max_idx.post_increment()
        };

        if idx.idx() as usize >= self.generations.len() {
            self.generations.resize(idx.idx() as usize + 1, Generation::default());
        }

        let gen = self.generations[idx.idx() as usize].activate();

        Entity::new(idx, gen)
    }

    /// Destroy the specified `Entity`.
    ///
    /// # Arguments
    ///
    /// * `entity` - The `Entity` to be destroyed.
    pub fn destroy(&mut self, entity: Entity) {
        let idx_usize: usize = entity.idx().into();
        self.generations[idx_usize].deactivate();
        self.free_idx.push(entity.idx());
    }

    /// Return `true` if there are no active entities
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the number of active entities.
    pub fn len(&self) -> usize {
        self.generations.iter().filter(|g| g.is_active()).count()
    }

    // Given an index, retrieve the currently used entity
    pub fn get<I: Into<Index>>(&self, index: I) -> Entity {
        let idx = index.into();
        let idx_usize: usize = idx.into();
        Entity::new(idx, self.generations[idx_usize])
    }

    pub fn try_get<I: Into<Index>>(&self, index: I) -> Option<Entity> {
        let idx = index.into();
        let idx_usize: usize = idx.into();
        self.generations.get(idx_usize).map(|gen| Entity::new(idx, *gen))
    }

    /// Create an iterator over all active entities.
    pub fn iter(&self) -> EntitiesIter {
        self.into_iter()
    }
}

impl Resource for Entities {}

impl SerializationProxy for Entities {}

impl<'a> IntoIterator for &'a Entities {
    type IntoIter = EntitiesIter<'a>;
    type Item = Entity;

    fn into_iter(self) -> Self::IntoIter {
        EntitiesIter {
            idx: 0,
            gens: &self.generations,
        }
    }
}

/// An iterator over all active entities.
pub struct EntitiesIter<'a> {
    /// Tracks the current index into the generations slice.
    idx: usize,
    /// Holds a reference to the current generations.
    gens: &'a [Generation],
}

impl<'a> Iterator for EntitiesIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.gens.len() {
            while self.idx < self.gens.len() {
                if self.gens[self.idx].is_active() {
                    let tmp = Entity::new(self.idx, self.gens[self.idx]);
                    self.idx += 1;
                    return Some(tmp);
                } else {
                    self.idx += 1;
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_default() {
        let _: Entities = Default::default();
    }

    #[test]
    fn entities_create() {
        let mut r = Entities::default();

        let _: Entity = r.create();
    }

    #[test]
    fn entities_destroy() {
        let mut r = Entities::default();
        let e: Entity = r.create();

        r.destroy(e);
    }

    #[test]
    fn entities_len() {
        let mut r = Entities::default();

        assert_eq!(r.len(), 0);
        let a = r.create();
        assert_eq!(r.len(), 1);
        let _b = r.create();
        assert_eq!(r.len(), 2);
        r.destroy(a);
        assert_eq!(r.len(), 1);
        let _c = r.create();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn entities_iter() {
        let mut r = Entities::default();
        let a = r.create();
        let b = r.create();
        let c = r.create();
        r.destroy(a);
        let d = r.create();
        let e = r.create();
        let f = r.create();
        r.destroy(c);

        let entities: Vec<Entity> = r.iter().collect();
        assert_eq!(entities, vec![d, b, e, f]);
    }

    #[test]
    fn entities_index_management() {
        let mut r = Entities::default();

        let a = r.create();
        assert_eq!(a.idx(), Index::new(0));
        assert_eq!(a.gen(), Generation::new(1));

        let b = r.create();
        assert_eq!(b.idx(), Index::new(1));
        assert_eq!(b.gen(), Generation::new(1));

        r.destroy(a);
        let c = r.create();
        assert_eq!(c.idx(), Index::new(0));
        assert_eq!(c.gen(), Generation::new(3));
    }
}
