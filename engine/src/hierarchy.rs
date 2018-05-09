use std::collections::HashMap;
use std::hash::Hash;
use daggy::{Dag, NodeIndex};

pub struct Hierarchy<K, V>
where
    K: Default + Clone + Eq + Hash,
{
    graph: Dag<K, V>,
    root: NodeIndex,
    node_index: HashMap<K, NodeIndex>,
}

impl<K, V> Default for Hierarchy<K, V>
where
    K: Default + Clone + Eq + Hash,
{
    fn default() -> Self {
        // Create the root node.
        let root_node: K = Default::default();
        let mut graph: Dag<K, V> = Default::default();
        let root_idx = graph.add_node(root_node);

        Hierarchy {
            graph: graph,
            root: root_idx,
            node_index: Default::default(),
        }
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Default + Clone + Eq + Hash,
{
    pub fn insert(&mut self, child: K, data: V) {
        let root = self.root;
        self.insert_child_internal(root, child, data)
    }

    pub fn insert_child(&mut self, parent: &K, child: K, data: V) -> Result<(), GraphError> {
        let parent_idx = self.get_node_index(parent)?;
        self.insert_child_internal(parent_idx, child, data);
        Ok(())
    }

    pub fn remove(&mut self, child: &K) -> Result<(), GraphError> {
        let child_idx = self.get_node_index(child)?;
        self.graph.remove_node(child_idx);
        self.rebuild_node_index();
        Ok(())
    }

    pub fn has(&self, child: &K) -> bool {
        self.node_index.contains_key(child)
    }

    pub fn update<F>(&mut self, merge_fn: &F) -> Result<(), GraphError>
    where
        for<'r> F: Fn(&'r K, Option<&'r V>) -> Option<V>,
    {
        Ok(())
    }

    fn insert_child_internal(&mut self, parent_idx: NodeIndex, child: K, data: V) {
        let (_, child_idx) = self.graph.add_child(parent_idx, data, child.clone());
        self.node_index.insert(child, child_idx);
    }

    fn rebuild_node_index(&mut self) {
        self.node_index.clear();
        for idx in self.graph.graph().node_indices() {
            let node = self.graph
                .node_weight(idx)
                .unwrap_or_else(|| unreachable!());
            self.node_index.insert(node.clone(), idx);
        }
    }

    fn get_node_index(&self, key: &K) -> Result<NodeIndex, GraphError> {
        self.node_index.get(key).cloned().ok_or(GraphError::KeyNotFound)
    }
}

#[derive(Debug, Fail)]
pub enum GraphError {
    #[fail(display = "The key was not found.")] KeyNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Clone, Hash, PartialEq, Eq)]
    struct MockKey(usize);

    #[test]
    fn default() {
        let _h: Hierarchy<MockKey, f32> = Default::default();
    }

    #[test]
    fn insertion_existence_and_deletion() {
        let mut h: Hierarchy<MockKey, f32> = Default::default();
        let key = MockKey(1);
        assert!(!h.has(&key));

        h.insert(key.clone(), 2.0);
        assert!(h.has(&key));

        let r = h.remove(&key);
        assert_ok!(r);
        assert!(!h.has(&key));

        let r = h.remove(&key);
        assert_err!(r);
    }

    #[test]
    fn update() {
        let mut h: Hierarchy<MockKey, f32> = Default::default();

        let r = h.update(&|id, parent_value| {
            if let Some(pv) = parent_value {
                Some(pv * 3.0)
            } else {
                Some(4.0)
            }
        });

        assert!(r.is_ok(), "{}", r.unwrap_err());
    }
}
