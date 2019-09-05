//! # Hierarchy
//!
//! Provides a hierarchical data structure, called `Hierarchy`. Its goal is to encode a
//! hierarchical relationships between identifiers (e.g. entities in an ECS) and a piece of data
//! associated with each identifier.
#![deny(missing_docs)]

#[macro_use]
extern crate failure;
extern crate daggy;
#[cfg(feature = "serde_support")]
extern crate serde;
#[cfg(feature = "serde_support")]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_test;

use daggy::{
    petgraph::{
        graph::{DefaultIx, Node},
        visit::{Bfs, Walker},
    },
    Dag, NodeIndex,
};
use std::{collections::HashMap, fmt, hash::Hash};

/// Given a set of identifying keys and corresponding data, `Hierarchy` allows users to establish
/// hierarchical relationships between individual instances of the data type.
///
/// # Example
///
/// ```
/// use hierarchy::Hierarchy;
///
/// #[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
/// struct Key(usize);
///
/// let mut h: Hierarchy<Key, f32> = Default::default();
///
/// h.insert(Key(1), 1.0);
/// h.insert(Key(2), 2.0);
/// h.insert_child(&Key(1), Key(3), 3.0).unwrap();
/// h.insert_child(&Key(1), Key(4), 4.0).unwrap();
/// h.insert_child(&Key(2), Key(5), 5.0).unwrap();
/// h.insert_child(&Key(2), Key(6), 6.0).unwrap();
///
/// h.update(&|_id, _value, parent_value| {
///     Some(parent_value + 1.0)
/// });
///
/// h.update(&|_id, value, parent_value| {
///     assert_eq!(*value, parent_value + 1.0);
///     Some(*value)
/// });
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Hierarchy<K, V>
where
    K: Eq + Hash,
{
    /// Holds the index of the root node.
    root_idx: NodeIndex,
    /// Provides an indexing relationship between keys and `NodeIndex` instances that in turn index
    /// into the directed acyclic graph (`Dag`).
    index: HashMap<K, NodeIndex>,
    /// Holds the directed acyclic graph of `HierNode`s.
    graph: Dag<HierNode<K, V>, ()>,
}

impl<K, V> Default for Hierarchy<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone + Default,
{
    /// Creates a default `Hierarchy` with just a root node.
    fn default() -> Self {
        let mut dag = Dag::new();
        let root_idx = dag.add_node(HierNode::root());

        Hierarchy {
            root_idx,
            index: HashMap::default(),
            graph: dag,
        }
    }
}

impl<K, V> fmt::Debug for Hierarchy<K, V>
where
    K: Eq + Hash,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Hierarchy(nodes: {}, edges: {})",
            self.graph.node_count(),
            self.graph.edge_count()
        )
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone + Default,
{
    /// Creates a new `Hierarchy`.
    pub fn new() -> Self {
        Hierarchy::default()
    }

    /// Removes all elements from the `Hierarchy` and then creates a new root node.
    pub fn clear(&mut self) {
        self.graph.clear();
        self.index.clear();
        self.root_idx = self.graph.add_node(HierNode::root());
    }

    /// Deletes the hierarchical node identified by the specified `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - The identifier of the node to be deleted.
    ///
    /// # Errors
    ///
    /// * `HierarchyError::CannotRemoveRootNode` if you try to remove the root node.
    /// * `HierarchyError::KeyNotFound` if `key` identifies no known node.
    pub fn remove(&mut self, key: &K) -> Result<(), HierarchyError> {
        let node_idx = self.get_index(key)?;
        self.graph.remove_node(node_idx);
        self.rebuild_index();
        Ok(())
    }

    /// Inserts a new node as child of the root node.
    ///
    /// # Arguments
    ///
    /// * `child` - The identifier of the new node.
    /// * `data` - The associated data.
    pub fn insert(&mut self, child: K, data: V) {
        let idx = self.root_idx;
        self.insert_child_internal(idx, child, data)
    }

    /// Inserts a new node as child of another node.
    ///
    /// # Arguments
    ///
    /// * `parent` - The identifier of the parent node.
    /// * `child` - The identifier of the new node.
    /// * `data` - The data associated with the new node.
    ///
    /// # Errors
    ///
    /// * `HierarchyError::KeyNotFound` if `parent` identifies no known node.
    pub fn insert_child(&mut self, parent: &K, child: K, data: V) -> Result<(), HierarchyError> {
        let parent_idx = self.get_index(parent)?;
        self.insert_child_internal(parent_idx, child, data);
        Ok(())
    }

    /// Returns `true` if the specified key is represented within the `Hierarchy`.
    ///
    /// # Arguments
    ///
    /// * `key` - The identifier to be verified for presence.
    pub fn has(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    /// Returns the number of nodes contained in the `Hierarchy` (excluding the internal root node).
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Returns `true` if the `Hierarchy` is empty, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Updates each node in breadth-first search order. Given a parent node's data, update the
    /// child nodes' data with the supplied closure. This allows users to establish hierarchical
    /// relationships between identifiers and associated data.
    ///
    /// # Arguments
    ///
    /// * `merge_fn` - A closure that takes a node identifier, its associated data and the parent's
    /// data as arguments. It may return new data that will overwrite the node's associated data.
    pub fn update<F>(&mut self, merge_fn: &F)
    where
        for<'r> F: Fn(&'r K, &'r V, &'r V) -> Option<V>,
    {
        // Traverse the tree in breadth-first search order and update each node.
        let mut bfs = Bfs::new(self.graph.graph(), self.root_idx);
        while let Some(nidx) = bfs.next(self.graph.graph()) {
            let mut parents = self.graph.parents(nidx);
            if let Some((_, parent_idx)) = parents.walk_next(&self.graph) {
                let parent_data = self
                    .graph
                    .node_weight(parent_idx)
                    .and_then(|n| n.0.as_ref().map(|(_, data)| data.clone()))
                    .unwrap_or_default();

                self.graph
                    .node_weight_mut(nidx)
                    .map(|n| n.update(&parent_data, merge_fn))
                    .expect(&format!("Could not find the child with index {:?}", nidx));
            }
        }
    }

    /// Returns an iterator over all pairs of identifier and data irrespective of hierarchical
    /// order.
    pub fn iter(&self) -> RawNodes<K, V> {
        RawNodes::new(self)
    }

    /// Insert a new node as a child of another node.
    fn insert_child_internal(&mut self, parent: NodeIndex, child: K, data: V) {
        let child_node = HierNode::new(child.clone(), data);
        let (_, child_idx) = self.graph.add_child(parent, (), child_node);
        self.index.insert(child, child_idx);
    }

    /// Returns the `NodeIndex` for a particular key.
    fn get_index(&self, key: &K) -> Result<NodeIndex, HierarchyError> {
        self.index.get(key).cloned().ok_or(HierarchyError::KeyNotFound)
    }

    /// Rebuilds the `Key`-`HierNode` index from the underlying `Graph`.
    fn rebuild_index(&mut self) {
        self.index.clear();
        for idx in self.graph.graph().node_indices() {
            let node = self.graph.node_weight(idx).unwrap_or_else(|| unreachable!());
            if let Some((ref key, _)) = node.0 {
                self.index.insert(key.clone(), idx);
            }
        }
    }
}

/// Each `HierNode` consists of an identifying key and the associated data.
#[derive(Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
struct HierNode<K, V>(Option<(K, V)>);

impl<K, V> HierNode<K, V>
where
    V: Clone,
{
    /// Creates a new `HierNode`.
    fn new(key: K, data: V) -> Self {
        HierNode(Some((key, data)))
    }

    /// Creates a new root node.
    fn root() -> Self {
        HierNode(None)
    }

    /// Given the parent node's data, update the current node's data with the supplied closure.
    /// This allows users to establish hierarchical relationships between instances of a type.
    /// As arguments, the closure will receive the current node's key and a reference to its parent
    /// node's data.
    fn update<F>(&mut self, parent_data: &V, merge_fn: &F)
    where
        for<'r> F: Fn(&'r K, &'r V, &'r V) -> Option<V>,
    {
        if let Some((ref k, ref mut v)) = self.0 {
            if let Some(data) = merge_fn(k, v, parent_data) {
                *v = data;
            }
        }
    }
}

/// Provides the ability to iterate over all `HierNode`s stored within a `Hierarchy`.
pub struct RawNodes<'a, K: 'a, V: 'a> {
    index: usize,
    data: &'a [Node<HierNode<K, V>, DefaultIx>],
}

impl<'a, K, V> RawNodes<'a, K, V>
where
    K: 'a + Eq + Hash,
    V: 'a,
{
    /// Creates a new iterator over the raw nodes of a `Hierarchy`.
    pub fn new(hierarchy: &'a Hierarchy<K, V>) -> Self {
        RawNodes {
            index: 0,
            data: hierarchy.graph.raw_nodes(),
        }
    }
}

impl<'a, K, V> Iterator for RawNodes<'a, K, V>
where
    K: 'a + Clone + Eq + Hash,
    V: 'a + Clone + Default,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            while self.index < self.data.len() && self.data[self.index].weight.0.is_none() {
                self.index += 1;
            }
            let idx = self.index;
            self.index += 1;
            let w = &self.data[idx].weight;
            w.0.as_ref().map(|&(ref k, ref v)| (k, v))
        } else {
            None
        }
    }
}

/// Describes possible errors when interacting with `Hierarchy`.
#[derive(Debug, Fail, PartialEq)]
pub enum HierarchyError {
    /// Returned when the requested key of type `K` was not found.
    #[fail(display = "The key was not found.")]
    KeyNotFound,
    /// Returned when the requested key was found more than once.
    #[fail(display = "The key was found more than once.")]
    MultipleKeysFound,
    /// Returned if the requested node was not found.
    #[fail(display = "The specified node was not found.")]
    NodeNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "serde_support")]
    use serde_test::{Token, assert_ser_tokens};

    #[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
    #[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
    struct MockKey(usize);

    #[test]
    fn default() {
        let _h: Hierarchy<MockKey, f32> = Default::default();
    }

    #[test]
    fn clear() {
        let mut h: Hierarchy<MockKey, f32> = Hierarchy::default();
        assert_eq!(h.len(), 0);
        assert!(h.is_empty());
        let key = MockKey(1);
        h.insert(key.clone(), 2.0);
        assert!(h.has(&key));
        assert_eq!(h.len(), 1);
        assert!(!h.is_empty());
        h.clear();
        assert!(!h.has(&key));
        assert_eq!(h.len(), 0);
        assert!(h.is_empty());
    }

    #[test]
    fn insertion_existence_and_deletion() {
        let mut h: Hierarchy<MockKey, f32> = Default::default();
        let key = MockKey(1);
        assert!(!h.has(&key));

        h.insert(key.clone(), 2.0);
        assert!(h.has(&key));
        assert!(h.remove(&key).is_ok());
        assert!(!h.has(&key));
        assert!(h.remove(&key).is_err());
    }

    #[test]
    fn update() {
        let mut h: Hierarchy<MockKey, f32> = Default::default();

        h.insert(MockKey(1), 1.0);
        h.insert(MockKey(2), 2.0);
        h.insert_child(&MockKey(1), MockKey(3), 3.0).unwrap();
        h.insert_child(&MockKey(1), MockKey(4), 4.0).unwrap();
        h.insert_child(&MockKey(2), MockKey(5), 5.0).unwrap();
        h.insert_child(&MockKey(2), MockKey(6), 6.0).unwrap();

        assert_eq!(h.len(), 6);
        assert!(!h.is_empty());

        h.update(&|_id, _value, parent_value| Some(parent_value + 1.0));
        h.update(&|_id, value, parent_value| {
            assert_eq!(*value, parent_value + 1.0);
            Some(*value)
        });
    }

    #[test]
    fn iteration() {
        let mut h: Hierarchy<MockKey, f32> = Default::default();

        h.insert(MockKey(1), 1.0);
        h.insert(MockKey(2), 2.0);
        h.insert_child(&MockKey(1), MockKey(3), 3.0).unwrap();
        h.insert_child(&MockKey(1), MockKey(4), 4.0).unwrap();
        h.insert_child(&MockKey(2), MockKey(5), 5.0).unwrap();
        h.insert_child(&MockKey(2), MockKey(6), 6.0).unwrap();

        // The node count will be the number of inserted nodes plus 1, because the root node will
        // always be in the hierarchy.
        assert_eq!(h.iter().count(), 6);
        assert_eq!(h.iter().map(|(_, v)| v).sum::<f32>(), 21.0);
    }

    #[cfg(feature = "serde_support")]
    #[test]
    fn ser_default() {
        let h: Hierarchy<MockKey, f32> = Hierarchy::default();

        assert_ser_tokens(&h, &[
            Token::Struct { name: "Hierarchy", len: 3 },
            Token::Str("root_idx"),
            Token::NewtypeStruct { name: "MockKey" },
            Token::U64(0),
            Token::Str("index"),
            Token::Map { len: Some(1) },
            Token::NewtypeStruct { name: "MockKey" },
            Token::U64(0),
            Token::MapEnd,
            Token::Str("graph"),
            Token::Struct { name: "Graph", len: 4 },
            Token::Str("nodes"),
            Token::Seq { len: Some(1) },
            Token::Struct { name: "HierNode", len: 2 },
            Token::Str("key"),
            Token::NewtypeStruct { name: "MockKey" },
            Token::U64(0),
            Token::Str("data"),
            Token::F32(0.0),
            Token::StructEnd,
            Token::SeqEnd,
            Token::Str("node_holes"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Str("edge_property"),
            Token::UnitVariant { name: "EdgeProperty", variant: "directed" },
            Token::Str("edges"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
            Token::StructEnd,
        ]);
    }
}
