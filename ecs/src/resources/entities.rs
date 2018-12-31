use super::Resource;

#[derive(Default, Debug)]
pub struct EntityResource {
    max_idx: Index,
    free_idx: Vec<Index>,
    generations: Vec<Generation>,
}

impl EntityResource {
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
}

impl Resource for EntityResource {}

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Index(u32);

impl Index {
    fn post_increment(&mut self) -> Index {
        let tmp = *self;
        self.0 += 1;
        tmp
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_resource_default() {
        let _: EntityResource = Default::default();
    }

    #[test]
    fn entity_resource_create() {
        let mut r = EntityResource::default();

        let _: Entity = r.create();
    }

    #[test]
    fn entity_resource_destroy() {
        let mut r = EntityResource::default();
        let e: Entity = r.create();

        r.destroy(e);
    }

    #[test]
    fn entity_resource_index_management() {
        let mut r = EntityResource::default();

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
