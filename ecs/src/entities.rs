use crate::resources::Resource;
use std::fmt;

#[derive(Default, Debug)]
pub struct Entities {
    max_idx: Index,
    free_idx: Vec<Index>,
    generations: Vec<Generation>,
}

impl Entities {
    pub fn create(&mut self) -> Entity {
        let idx = if let Some(idx) = self.free_idx.pop() {
            idx
        } else {
            self.max_idx.post_increment()
        };

        if idx.0 as usize >= self.generations.len() {
            self.generations.resize(idx.0 as usize + 1, Generation::default());
        }

        let gen = self.generations[idx.0 as usize].activate();

        Entity { idx, gen }
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.generations[entity.idx.0 as usize].deactivate();
        self.free_idx.push(entity.idx);
    }

    pub fn len(&self) -> usize {
        self.generations.iter().filter(|g| g.is_active()).count()
    }

    pub fn iter(&self) -> EntitiesIter {
        EntitiesIter { idx: 0, gens: &self.generations }
    }
}

impl Resource for Entities {}

pub struct EntitiesIter<'a> {
    idx: usize,
    gens: &'a [Generation],
}

impl<'a> Iterator for EntitiesIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.gens.len() {
            while self.idx < self.gens.len() {
                if self.gens[self.idx].is_active() {
                    let tmp = Entity { idx: Index(self.idx as u32), gen: self.gens[self.idx] };
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    idx: Index,
    gen: Generation,
}

impl Entity {
    #[cfg(test)]
    pub fn new(idx: u32, gen: u32) -> Entity {
        Entity {
            idx: Index(idx),
            gen: Generation(gen),
        }
    }

    pub fn idx(&self) -> u32 {
        self.idx.0
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Entity({}, {})", self.idx, self.gen)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Index(u32);

impl Index {
    fn post_increment(&mut self) -> Index {
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Generation(u32);

impl Generation {
    fn activate(&mut self) -> Generation {
        if !self.is_active() {
            self.0 += 1;
            *self
        } else {
            panic!("Attempted to activate an active generation")
        }
    }

    fn deactivate(&mut self) -> Generation {
        if self.is_active() {
            self.0 += 1;
            *self
        } else {
            panic!("Attempted to deactivate an inactive generation")
        }
    }

    fn is_active(&self) -> bool {
        self.0 % 2 == 1
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
        assert_eq!(a.idx, Index(0));
        assert_eq!(a.gen, Generation(1));

        let b = r.create();
        assert_eq!(b.idx, Index(1));
        assert_eq!(b.gen, Generation(1));

        r.destroy(a);
        let c = r.create();
        assert_eq!(c.idx, Index(0));
        assert_eq!(c.gen, Generation(3));
    }
}
