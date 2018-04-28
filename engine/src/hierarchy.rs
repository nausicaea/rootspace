use std::hash::Hash;
use std::marker::PhantomData;
use daggy::{Dag, NodeIndex};

pub struct Hierarchy<K, V> {
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

impl<K, V> Default for Hierarchy<K, V> {
    fn default() -> Self {
        Hierarchy {
            phantom_k: Default::default(),
            phantom_v: Default::default(),
        }
    }
}

impl<K, V> Hierarchy<K, V> {
    pub fn insert(&mut self, _child: K, _data: V) {
    }
    pub fn update<F>(&mut self, merge_fn: &F) -> Result<(), GraphError>
    where
        for<'r> F: Fn(&'r K, Option<&'r V>) -> Option<V>,
    {
        Ok(())
    }
}

pub struct HierNode;

pub enum GraphError {
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockKey(usize);

    #[test]
    fn default() {
        let _h: Hierarchy<MockKey, f32> = Default::default();
    }

    #[test]
    fn insert() {
        let mut h: Hierarchy<MockKey, f32> = Default::default();

        h.insert(MockKey(1), 2.0);
    }

    #[test]
    fn update() {
        let mut h: Hierarchy<MockKey, f32> = Default::default();

        let r = h.update(&|id, parent_value| {
            if let Some(ref pv) = parent_value {
                Some(pv * 3.0)
            } else {
                Some(4.0)
            }
        });

        assert!(r.is_ok(), "{}", r.unwrap_err());
    }
}
