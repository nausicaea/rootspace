//! # Hierarchy
//!
//! Provides a hierarchical data structure, called `Hierarchy`. Its goal is to encode a
//! hierarchical relationships between identifiers (e.g. entities in an ECS) and a piece of data
//! associated with each identifier.
#![deny(missing_docs)]

use daggy::{
    petgraph::{
        graph::{DefaultIx, Node},
        visit::{Bfs, Walker},
    },
    Dag, NodeIndex,
};
#[cfg(any(test, feature = "serde_support"))]
use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::{collections::HashMap, fmt, hash::Hash, marker::PhantomData};
use thiserror::Error;

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
#[cfg_attr(any(test, feature = "serde_support"), derive(Serialize))]
#[derive(Clone)]
pub struct Hierarchy<K, V>
where
    K: Eq + Hash,
{
    /// Holds the index of the root node.
    root_idx: NodeIndex,
    /// Provides an entity relationship between keys and `NodeIndex` instances that in turn index
    /// into the directed acyclic graph (`Dag`).
    #[cfg_attr(any(test, feature = "serde_support"), serde(skip))]
    index: HashMap<K, NodeIndex>,
    /// Holds the directed acyclic graph of `HierNode`s.
    graph: Dag<HierarchyNode<K, V>, ()>,
}

impl<K, V> Hierarchy<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new `Hierarchy`.
    pub fn new() -> Self {
        Hierarchy::default()
    }

    /// Removes all elements from the `Hierarchy` and then creates a new root node.
    pub fn clear(&mut self) {
        self.graph.clear();
        self.index.clear();
        self.root_idx = self.graph.add_node(HierarchyNode::root());
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

    /// Returns an iterator over all pairs of identifier and data irrespective of hierarchical
    /// order.
    pub fn iter(&self) -> RawNodes<K, V> {
        self.into_iter()
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Eq + Hash + Clone,
{
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
    pub fn remove(&mut self, key: K) -> Result<(), HierarchyError> {
        let node_idx = self.get_index(&key)?;
        self.graph.remove_node(node_idx);
        self.rebuild_index();
        Ok(())
    }

    /// Rebuilds the `Key`-`HierNode` index from the underlying `Graph`.
    pub fn rebuild_index(&mut self) {
        self.index.clear();
        for idx in self.graph.graph().node_indices() {
            let node = self.graph.node_weight(idx).unwrap_or_else(|| unreachable!());
            if let Some((ref key, _)) = node.0 {
                self.index.insert(key.clone(), idx);
            }
        }
    }

    /// Returns the `NodeIndex` for a particular key.
    fn get_index(&self, key: &K) -> Result<NodeIndex, HierarchyError> {
        self.index.get(key).cloned().ok_or(HierarchyError::KeyNotFound)
    }

    /// Insert a new node as a child of another node.
    fn insert_child_internal(&mut self, parent: NodeIndex, child: K, data: V) {
        let child_node = HierarchyNode::new(child.clone(), data);
        let (_, child_idx) = self.graph.add_child(parent, (), child_node);
        self.index.insert(child, child_idx);
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone + Default,
{
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
                    .and_then(|n| n.0.as_ref().map(|(_, value)| value.clone()))
                    .unwrap_or_default();

                self.graph
                    .node_weight_mut(nidx)
                    .map(|n| n.update(&parent_data, merge_fn))
                    .unwrap_or_else(|| panic!("Could not find the child with index {:?}", nidx));
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a Hierarchy<K, V>
where
    K: 'a + Eq + Hash,
    V: 'a,
{
    type IntoIter = RawNodes<'a, K, V>;
    type Item = <RawNodes<'a, K, V> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        RawNodes::new(self)
    }
}

impl<K, V> Default for Hierarchy<K, V>
where
    K: Eq + Hash,
{
    /// Creates a default `Hierarchy` with just a root node.
    fn default() -> Self {
        let mut dag = Dag::new();
        let root_idx = dag.add_node(HierarchyNode::root());

        Hierarchy {
            root_idx,
            index: HashMap::default(),
            graph: dag,
        }
    }
}

impl<K, V> PartialEq<Hierarchy<K, V>> for Hierarchy<K, V>
where
    K: PartialEq + Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        if !self.index.keys().all(|lhsv| rhs.index.keys().any(|rhsv| lhsv == rhsv)) {
            return false;
        }

        let lhs_bfs = Bfs::new(self.graph.graph(), self.root_idx).iter(self.graph.graph());
        let rhs_bfs = Bfs::new(rhs.graph.graph(), rhs.root_idx).iter(rhs.graph.graph());

        lhs_bfs
            .zip(rhs_bfs)
            .all(|(lhs_nidx, rhs_nidx)| self.graph.node_weight(lhs_nidx) == rhs.graph.node_weight(rhs_nidx))
    }
}

impl<K, V> Eq for Hierarchy<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
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

#[cfg(any(test, feature = "serde_support"))]
impl<'de, K, V> Deserialize<'de> for Hierarchy<K, V>
where
    K: Clone + Eq + Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["root_idx", "graph"];

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            RootIdx,
            Graph,
        }

        struct HierarchyVisitor<K, V>(PhantomData<(K, V)>);

        impl<K, V> Default for HierarchyVisitor<K, V> {
            fn default() -> Self {
                HierarchyVisitor(PhantomData::default())
            }
        }

        impl<'de, K, V> Visitor<'de> for HierarchyVisitor<K, V>
        where
            K: Clone + Eq + Hash + Deserialize<'de>,
            V: Deserialize<'de>,
        {
            type Value = Hierarchy<K, V>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a struct Hierarchy")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut root_idx = None;
                let mut graph = None;

                while let Some(key) = access.next_key()? {
                    match key {
                        Field::RootIdx => {
                            if root_idx.is_some() {
                                return Err(de::Error::duplicate_field("root_idx"));
                            }
                            root_idx = Some(access.next_value()?);
                        }
                        Field::Graph => {
                            if graph.is_some() {
                                return Err(de::Error::duplicate_field("graph"));
                            }
                            graph = Some(access.next_value()?);
                        }
                    }
                }

                let root_idx = root_idx.ok_or_else(|| de::Error::missing_field("root_idx"))?;
                let graph = graph.ok_or_else(|| de::Error::missing_field("graph"))?;

                let mut h = Hierarchy {
                    root_idx,
                    index: HashMap::default(),
                    graph,
                };

                h.rebuild_index();

                Ok(h)
            }
        }

        de.deserialize_struct("Hierarchy", FIELDS, HierarchyVisitor::<K, V>::default())
    }
}

/// Each `HierNode` consists of an identifying key and the associated data.
#[cfg_attr(any(test, feature = "serde_support"), derive(Serialize, Deserialize))]
#[cfg_attr(any(test, feature = "serde_support"), serde(transparent))]
#[derive(Clone, PartialEq, Eq)]
struct HierarchyNode<K, V>(Option<(K, V)>);

impl<K, V> HierarchyNode<K, V> {
    /// Creates a new `HierNode`.
    fn new(key: K, value: V) -> Self {
        HierarchyNode(Some((key, value)))
    }

    /// Creates a new root node.
    fn root() -> Self {
        HierarchyNode(None)
    }

    /// Given the parent node's data, update the current node's data with the supplied closure.
    /// This allows users to establish hierarchical relationships between instances of a type.
    /// As arguments, the closure will receive the current node's key and a reference to its parent
    /// node's data.
    fn update<F>(&mut self, parent_data: &V, merge_fn: &F)
    where
        for<'r> F: Fn(&'r K, &'r V, &'r V) -> Option<V>,
    {
        if let Some((ref key, ref mut value)) = self.0 {
            if let Some(data) = merge_fn(key, value, parent_data) {
                *value = data;
            }
        }
    }
}

/// Provides the ability to iterate over all `HierNode`s stored within a `Hierarchy`.
pub struct RawNodes<'a, K: 'a, V: 'a> {
    index: usize,
    data: &'a [Node<HierarchyNode<K, V>, DefaultIx>],
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
    K: 'a + Eq + Hash,
    V: 'a,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            // Skip those nodes that don't have any data associated with them
            while self.index < self.data.len() && self.data[self.index].weight.0.is_none() {
                self.index += 1;
            }

            // If none of the nodes contain data, return
            if self.index >= self.data.len() {
                return None;
            }

            // Retrieve the current node's data
            let w = &self.data[self.index].weight;
            self.index += 1;
            w.0.as_ref()
                .map(|&(ref key, ref value)| (key, value))
        } else {
            None
        }
    }
}

/// Describes possible errors when interacting with `Hierarchy`.
#[derive(Debug, Error, PartialEq)]
pub enum HierarchyError {
    /// Returned when the requested key of type `K` was not found.
    #[error("The key was not found")]
    KeyNotFound,
    /// Returned when the requested key was found more than once.
    #[error("The key was found more than once")]
    MultipleKeysFound,
    /// Returned if the requested node was not found.
    #[error("The specified node was not found")]
    NodeNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_ser_tokens, Token};
    use std::{num::ParseIntError, str::FromStr};

    #[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
    struct TestKey(u64);

    impl fmt::Display for TestKey {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl FromStr for TestKey {
        type Err = ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let idx: u64 = s.parse()?;
            Ok(TestKey(idx))
        }
    }

    #[test]
    fn default() {
        let _h: Hierarchy<TestKey, f32> = Default::default();
    }

    #[test]
    fn clear() {
        let mut h: Hierarchy<TestKey, f32> = Hierarchy::default();
        assert_eq!(h.len(), 0);
        assert!(h.is_empty());
        let key = TestKey(1);
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
        let mut h: Hierarchy<TestKey, f32> = Default::default();
        let key = TestKey(1);
        assert!(!h.has(&key));

        h.insert(key.clone(), 2.0);
        assert!(h.has(&key));
        assert!(h.remove(key).is_ok());
        assert!(!h.has(&key));
        assert!(h.remove(key).is_err());
    }

    #[test]
    fn update() {
        let mut h: Hierarchy<TestKey, f32> = Default::default();

        h.insert(TestKey(1), 1.0);
        h.insert(TestKey(2), 2.0);
        h.insert_child(&TestKey(1), TestKey(3), 3.0).unwrap();
        h.insert_child(&TestKey(1), TestKey(4), 4.0).unwrap();
        h.insert_child(&TestKey(2), TestKey(5), 5.0).unwrap();
        h.insert_child(&TestKey(2), TestKey(6), 6.0).unwrap();

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
        let mut h: Hierarchy<TestKey, f32> = Default::default();

        h.insert(TestKey(1), 1.0);
        h.insert(TestKey(2), 2.0);
        h.insert_child(&TestKey(1), TestKey(3), 3.0).unwrap();
        h.insert_child(&TestKey(1), TestKey(4), 4.0).unwrap();
        h.insert_child(&TestKey(2), TestKey(5), 5.0).unwrap();
        h.insert_child(&TestKey(2), TestKey(6), 6.0).unwrap();

        // The node count will be the number of inserted nodes plus 1, because the root node will
        // always be in the hierarchy.
        assert_eq!(h.iter().count(), 6);
        assert_eq!(h.iter().map(|(_, v)| v).sum::<f32>(), 21.0);
    }

    #[test]
    fn partial_eq() {
        let mut h: Hierarchy<TestKey, f32> = Default::default();

        h.insert(TestKey(1), 1.0);
        h.insert(TestKey(2), 2.0);

        let mut i: Hierarchy<TestKey, f32> = Default::default();

        i.insert(TestKey(1), 1.0);
        i.insert(TestKey(2), 2.0);

        let mut j: Hierarchy<TestKey, f32> = Default::default();

        j.insert(TestKey(1), 1.0);
        j.insert(TestKey(2), 3.0);

        assert_eq!(h, i);
        assert_ne!(h, j);
        assert_ne!(i, j);
    }

    #[test]
    fn serde() {
        let mut h: Hierarchy<TestKey, f32> = Default::default();

        h.insert(TestKey(1), 1.0);
        h.insert(TestKey(2), 2.0);

        assert_ser_tokens(
            &h,
            &[
                Token::Struct {
                    name: "Hierarchy",
                    len: 2,
                },
                Token::Str("root_idx"),
                Token::U32(0),
                Token::Str("graph"),
                Token::Struct { name: "Graph", len: 4 },
                Token::Str("nodes"),
                Token::Seq { len: Some(3) },
                Token::NewtypeStruct { name: "HierNode" },
                Token::None,
                Token::NewtypeStruct { name: "HierNode" },
                Token::Some,
                Token::Tuple { len: 2 },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(1),
                Token::F32(1.0),
                Token::TupleEnd,
                Token::NewtypeStruct { name: "HierNode" },
                Token::Some,
                Token::Tuple { len: 2 },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(2),
                Token::F32(2.0),
                Token::TupleEnd,
                Token::SeqEnd,
                Token::Str("node_holes"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("edge_property"),
                Token::UnitVariant {
                    name: "EdgeProperty",
                    variant: "directed",
                },
                Token::Str("edges"),
                Token::Seq { len: Some(2) },
                Token::Some,
                Token::Tuple { len: 3 },
                Token::U32(0),
                Token::U32(1),
                Token::Unit,
                Token::TupleEnd,
                Token::Some,
                Token::Tuple { len: 3 },
                Token::U32(0),
                Token::U32(2),
                Token::Unit,
                Token::TupleEnd,
                Token::SeqEnd,
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
