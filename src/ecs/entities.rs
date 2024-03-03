//! Provides facilities for reasoning about entities (e.g. objects) within a world.

use std::iter::FusedIterator;

use serde::{Deserialize, Serialize};

use super::{
    entity::{generation::Generation, index::Index, Entity},
    resource::Resource,
    with_dependencies::WithDependencies,
};

/// The `Entities` resource keeps track of all entities.
#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entities {
    /// Stores the highest assigned `Entity` index plus one.
    max_idx: Index,
    /// Stores all previously assigned `Entity` indices that are now available again.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    free_idx: Vec<Index>,
    /// Stores the generations of each `Entity`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
    pub fn destroy<I: Into<Index>>(&mut self, entity: I) {
        let idx = entity.into();
        let idx_usize: usize = idx.into();
        self.generations[idx_usize].deactivate();
        self.free_idx.push(idx);
    }

    /// Return `true` if there are no active entities
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the number of active entities.
    pub fn len(&self) -> usize {
        self.generations.iter().filter(|g| g.is_active()).count()
    }

    /// Given an index, retrieve the currently used entity
    pub fn get<I: Into<Index>>(&self, index: I) -> Option<Entity> {
        let idx = index.into();
        let idx_usize: usize = idx.into();
        self.generations.get(idx_usize).map(|gen| Entity::new(idx, *gen))
    }

    /// Create an iterator over all active entities.
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }
}

impl Resource for Entities {}

impl<D> WithDependencies<D> for Entities {
    async fn with_deps(_: &D) -> Result<Self, anyhow::Error> {
        Ok(Entities::default())
    }
}

impl<'a> IntoIterator for &'a Entities {
    type IntoIter = Iter<'a>;
    type Item = Entity;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            idx: 0,
            gens: &self.generations,
        }
    }
}

/// An iterator over all active entities.
pub struct Iter<'a> {
    /// Tracks the current index into the generations slice.
    idx: usize,
    /// Holds a reference to the current generations.
    gens: &'a [Generation],
}

impl<'a> Iterator for Iter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.gens.len() {
            return None;
        }

        while self.idx < self.gens.len() {
            if self.gens[self.idx].is_active() {
                let tmp = Entity::new(self.idx, self.gens[self.idx]);
                self.idx += 1;
                return Some(tmp);
            } else {
                self.idx += 1;
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_length = self.gens.len().saturating_sub(self.idx);

        (remaining_length, Some(remaining_length))
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> FusedIterator for Iter<'a> {}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;
    use crate::ecs::{registry::End, registry::ResourceRegistry, world::World};
    use crate::Reg;

    #[test]
    fn entities_reg_macro() {
        type _RR = Reg![Entities];
    }

    #[test]
    fn entities_resource_registry() {
        let _rr = ResourceRegistry::push(End, Entities::default());
    }

    #[tokio::test]
    async fn entities_world() {
        let _w = World::with_dependencies::<Reg![Entities], Reg![], Reg![], (), _>(&())
            .await
            .unwrap();
    }

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

    #[test]
    fn serde() {
        let mut es = Entities::default();
        let _e1 = es.create();
        let e2 = es.create();
        let _e3 = es.create();
        es.destroy(e2);

        assert_tokens(
            &es,
            &[
                Token::Struct {
                    name: "Entities",
                    len: 3,
                },
                Token::Str("max_idx"),
                Token::U32(3),
                Token::Str("free_idx"),
                Token::Seq { len: Some(1) },
                Token::U32(1),
                Token::SeqEnd,
                Token::Str("generations"),
                Token::Seq { len: Some(3) },
                Token::U32(1),
                Token::U32(2),
                Token::U32(1),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}
