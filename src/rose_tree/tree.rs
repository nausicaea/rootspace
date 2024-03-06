#![allow(dead_code)]

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    iter::FusedIterator,
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "K: Ord + std::hash::Hash + serde::Serialize, V: serde::Serialize",
    deserialize = "K: Ord + std::hash::Hash + for<'r> serde::Deserialize<'r>, V: for<'r> serde::Deserialize<'r>"
))]
#[derive(Debug)]
pub struct Tree<K, V> {
    pub(super) edges: HashMap<K, Vec<K>>,
    pub(super) parents: BTreeMap<K, Option<K>>,
    pub(super) nodes: HashMap<K, V>,
}

impl<K, V> Tree<K, V> {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.parents.clear();
        self.nodes.clear();
    }
}

impl<K, V> Tree<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn has_children<J: AsRef<K>>(&self, key: J) -> bool {
        self.edges.get(key.as_ref()).map(|e| !e.is_empty()).unwrap_or(false)
    }
}

impl<K, V> Tree<K, V>
where
    K: Clone,
{
    pub fn bfs_iter(&self) -> BfsIter<K, V> {
        BfsIter::new(self)
    }

    pub fn dfs_iter(&self) -> DfsIter<K, V> {
        DfsIter::new(self)
    }

    pub fn ancestors<J: AsRef<K>>(&self, key: J) -> AncestorsIter<K, V> {
        AncestorsIter::new(self, key.as_ref())
    }
}

impl<K, V> Tree<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn contains_key<J: AsRef<K>>(&self, key: J) -> bool {
        self.nodes.contains_key(key.as_ref())
    }

    pub fn get<J: AsRef<K>>(&self, key: J) -> Option<&V> {
        self.nodes.get(key.as_ref())
    }
}

impl<K, V> Tree<K, V>
where
    K: Clone + Ord + std::hash::Hash,
{
    pub fn insert<I: Into<K>>(&mut self, key: I, value: V) -> bool {
        let key = key.into();

        if self.nodes.contains_key(&key) {
            return false;
        }

        self.parents.insert(key.clone(), None);
        self.nodes.insert(key, value);

        true
    }

    pub fn insert_child<J: AsRef<K>, I: Into<K>>(&mut self, parent: J, key: I, value: V) -> bool {
        let parent = parent.as_ref();
        let key = key.into();

        if parent == &key {
            return false;
        }

        if self.nodes.contains_key(&key) {
            return false;
        }

        if !self.nodes.contains_key(parent) {
            panic!("The parent node does not exist");
        }

        self.edges
            .entry(parent.clone())
            .and_modify(|e| e.push(key.clone()))
            .or_insert_with(|| vec![key.clone()]);
        self.parents.insert(key.clone(), Some(parent.clone()));
        self.nodes.insert(key, value);

        true
    }

    pub fn remove<J: AsRef<K>>(&mut self, key: J) -> bool {
        let key = key.as_ref();

        if !self.nodes.contains_key(key) {
            return false;
        }

        // Push the target node onto the removal queue
        let mut queue: VecDeque<K> = VecDeque::new();
        queue.push_back(key.clone());

        while let Some(k) = queue.pop_front() {
            // Remove all edges of the current node
            queue.extend(
                self.edges
                    .remove(&k)
                    .into_iter()
                    .flat_map(|children| children.into_iter()),
            );

            // Remove all dangling edges
            for children in self.edges.values_mut() {
                children.retain(|c| c != &k);
            }

            // Remove the current node from parents
            self.parents.retain(|c, _| c != &k);

            // Remove the current node's value
            self.nodes.remove(&k);
        }

        true
    }
}

impl<K, V> Default for Tree<K, V>
where
    K: Ord,
{
    fn default() -> Self {
        Tree {
            edges: HashMap::default(),
            parents: BTreeMap::default(),
            nodes: HashMap::default(),
        }
    }
}

impl<K, V> Display for Tree<K, V>
where
    K: Display + Eq + PartialEq + Hash,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn display_rec<K: Display + Eq + PartialEq + Hash>(
            f: &mut Formatter,
            edges: &HashMap<K, Vec<K>>,
            key: &K,
            depth: usize,
        ) -> std::fmt::Result {
            // Print the current node
            writeln!(f, "{:indent$}{}", "", key, indent = depth * 4)?;

            // Recursively display child nodes
            if let Some(children) = edges.get(key) {
                for child in children {
                    display_rec(f, edges, child, depth + 1)?;
                }
            }

            Ok(())
        }

        let root_nodes = self.parents.iter().filter(|(_, p)| p.is_none()).map(|(n, _)| n);

        for root_node in root_nodes {
            display_rec(f, &self.edges, root_node, 0)?;
        }

        Ok(())
    }
}

impl<K, V> PartialEq for Tree<K, V>
where
    K: Eq + std::hash::Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.edges.eq(&other.edges) && self.nodes.eq(&other.nodes) && self.parents.eq(&other.parents)
    }
}

pub struct AncestorsIter<'a, K, V> {
    key: Option<K>,
    hier: &'a Tree<K, V>,
}

impl<'a, K, V> AncestorsIter<'a, K, V>
where
    K: Clone,
{
    fn new(hier: &'a Tree<K, V>, key: &K) -> Self {
        AncestorsIter {
            key: Some(key.clone()),
            hier,
        }
    }
}

impl<'a, K, V> Iterator for AncestorsIter<'a, K, V>
where
    K: Clone + Ord + Eq + std::hash::Hash,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.key.as_ref()?;

        if let Some(node) = self.hier.nodes.get_key_value(key) {
            self.key = self.hier.parents.get(key).cloned().flatten();
            Some(node)
        } else {
            self.key = None;
            None
        }
    }
}

impl<'a, K, V> FusedIterator for AncestorsIter<'a, K, V> where K: Clone + Ord + Eq + std::hash::Hash {}

pub struct BfsIter<'a, K, V> {
    queue: VecDeque<K>,
    hier: &'a Tree<K, V>,
}

impl<'a, K, V> BfsIter<'a, K, V>
where
    K: Clone,
{
    fn new(hier: &'a Tree<K, V>) -> Self {
        BfsIter {
            queue: hier
                .parents
                .iter()
                .filter(|(_, p)| p.is_none())
                .map(|(k, _)| k)
                .cloned()
                .collect(),
            hier,
        }
    }
}

impl<'a, K, V> Iterator for BfsIter<'a, K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.queue.pop_front() {
            self.queue.extend(
                self.hier
                    .edges
                    .get(&next_node)
                    .iter()
                    .flat_map(|children| children.iter().cloned()),
            );

            self.hier.nodes.get_key_value(&next_node)
        } else {
            None
        }
    }
}

pub struct DfsIter<'a, K, V> {
    stack: Vec<K>,
    hier: &'a Tree<K, V>,
}

impl<'a, K, V> DfsIter<'a, K, V>
where
    K: Clone,
{
    fn new(hier: &'a Tree<K, V>) -> Self {
        DfsIter {
            stack: hier
                .parents
                .iter()
                .filter(|(_, p)| p.is_none())
                .map(|(k, _)| k)
                .rev()
                .cloned()
                .collect(),
            hier,
        }
    }
}

impl<'a, K, V> Iterator for DfsIter<'a, K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.stack.pop() {
            self.stack.extend(
                self.hier
                    .edges
                    .get(&next_node)
                    .iter()
                    .flat_map(|children| children.iter().rev().cloned()),
            );

            self.hier.nodes.get_key_value(&next_node)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

    #[cfg(feature = "test-slow")]
    use serde_test::{assert_tokens, Token};

    use super::{BfsIter, DfsIter, Tree};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
    struct Tk(usize);

    impl AsRef<Tk> for Tk {
        fn as_ref(&self) -> &Tk {
            self
        }
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Tv(&'static str);

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Tvm(usize);

    impl Default for Tvm {
        fn default() -> Self {
            Tvm(1)
        }
    }

    impl<'a> Mul<&'a Tvm> for &'a Tvm {
        type Output = Tvm;

        fn mul(self, rhs: &'a Tvm) -> Self::Output {
            Tvm(self.0 * rhs.0)
        }
    }

    #[test]
    fn impl_default() {
        let _: Tree<Tk, Tv> = Default::default();
    }

    #[test]
    fn insert() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("Hello, World!"));
        rt.insert(Tk(1), Tv("Good night, World!"));
    }

    #[test]
    fn insert_child() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("Hello, World!"));
        rt.insert_child(&Tk(0), Tk(1), Tv("Good night, World!"));
        rt.insert_child(&Tk(0), Tk(2), Tv("Data for TestKey2"));
    }

    #[test]
    #[should_panic]
    fn insert_child_parent_does_not_exist() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert_child(&Tk(0), Tk(1), Tv("Good night, World!"));
    }

    #[test]
    fn no_cycles() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        assert!(rt.insert(Tk(0), Tv("0")));
        assert_eq!(rt.get(&Tk(0)), Some(&Tv("0")));
        assert!(!rt.insert(Tk(0), Tv("Some other value")));
        assert_eq!(rt.get(&Tk(0)), Some(&Tv("0")));
        assert!(rt.insert(Tk(1), Tv("1")));
        assert_eq!(rt.get(&Tk(1)), Some(&Tv("1")));
        assert!(!rt.insert_child(&Tk(1), Tk(0), Tv("Zero")));
        assert_eq!(rt.get(&Tk(0)), Some(&Tv("0")));
        assert!(rt.remove(&Tk(0)));
        assert!(rt.insert_child(&Tk(1), Tk(0), Tv("Zero")));
        assert_eq!(rt.get(&Tk(0)), Some(&Tv("Zero")));
        assert!(!rt.insert_child(&Tk(3), Tk(3), Tv("Self-cycle")));
        assert_eq!(rt.get(&Tk(3)), None);
    }

    #[test]
    fn has_children() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("A"));
        rt.insert(Tk(1), Tv("B"));
        rt.insert_child(&Tk(1), Tk(3), Tv("C"));
        rt.insert_child(&Tk(3), Tk(5), Tv("D"));
        rt.insert_child(&Tk(1), Tk(4), Tv("E"));

        assert!(!rt.has_children(&Tk(0)));
        assert!(rt.has_children(&Tk(1)));
    }

    #[test]
    fn get() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("Hello, World!"));
        rt.insert_child(&Tk(0), Tk(1), Tv("Good night, World!"));

        assert_eq!(rt.get(&Tk(0)), Some(&Tv("Hello, World!")));
        assert_eq!(rt.get(&Tk(1)), Some(&Tv("Good night, World!")));
        assert!(rt.get(&Tk(2)).is_none());
    }

    #[test]
    fn remove() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("Hello, World!"));
        rt.insert_child(&Tk(0), Tk(2), Tv("Not my tree"));
        rt.insert(Tk(1), Tv("Good night, World!"));
        rt.insert_child(&Tk(1), Tk(3), Tv("I am the shadow of the night"));
        rt.insert_child(&Tk(3), Tk(5), Tv("Hissssssss"));
        rt.insert_child(&Tk(1), Tk(4), Tv("I am the light in the night"));

        assert!(rt.remove(&Tk(1)));
        assert!(rt.contains_key(&Tk(0)));
        assert!(rt.contains_key(&Tk(2)));
        assert!(!rt.contains_key(&Tk(1)));
        assert!(!rt.contains_key(&Tk(3)));
        assert!(!rt.contains_key(&Tk(4)));
        assert!(!rt.contains_key(&Tk(5)));
        assert!(!rt.remove(&Tk(1)));
    }

    #[test]
    fn is_empty() {
        let mut rt: Tree<(), ()> = Tree::default();

        assert!(rt.is_empty());
        rt.insert((), ());
        assert!(!rt.is_empty());
    }

    #[test]
    fn len() {
        let mut rt: Tree<Tk, Tv> = Tree::default();

        assert_eq!(rt.len(), 0);
        rt.insert(Tk(0), Tv("TestKey(0)"));
        assert_eq!(rt.len(), 1);
        rt.insert_child(&Tk(0), Tk(1), Tv("TestKey(1)"));
        assert_eq!(rt.len(), 2);
    }

    #[test]
    fn contains() {
        let mut rt: Tree<Tk, Tv> = Tree::default();

        assert!(!rt.contains_key(&Tk(0)));
        rt.insert(Tk(0), Tv("TestKey(0)"));
        assert!(rt.contains_key(&Tk(0)));
        rt.insert_child(&Tk(0), Tk(1), Tv("TestKey(1)"));
        assert!(rt.contains_key(&Tk(1)));
    }

    #[test]
    fn clear() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("TestKey(0)"));
        rt.insert_child(&Tk(0), Tk(1), Tv("TestKey(1)"));

        rt.clear();
        assert!(rt.is_empty());
    }

    #[test]
    fn bfs_iter() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("Hello, World!"));
        rt.insert_child(&Tk(0), Tk(2), Tv("Not my tree"));
        rt.insert(Tk(1), Tv("Good night, World!"));
        rt.insert_child(&Tk(1), Tk(3), Tv("I am the shadow of the night"));
        rt.insert_child(&Tk(3), Tk(5), Tv("Hissssssss"));
        rt.insert_child(&Tk(1), Tk(4), Tv("I am the light in the night"));

        let bfsiter = BfsIter::new(&rt);
        let keys: Vec<Tk> = bfsiter.map(|n| n.0.clone()).collect();
        assert_eq!(keys, &[Tk(0), Tk(1), Tk(2), Tk(3), Tk(4), Tk(5)]);
    }

    #[test]
    fn dfs_iter() {
        let mut rt: Tree<Tk, Tv> = Tree::default();
        rt.insert(Tk(0), Tv("Hello, World!"));
        rt.insert_child(&Tk(0), Tk(2), Tv("Not my tree"));
        rt.insert(Tk(1), Tv("Good night, World!"));
        rt.insert_child(&Tk(1), Tk(3), Tv("I am the shadow of the night"));
        rt.insert_child(&Tk(3), Tk(5), Tv("Hissssssss"));
        rt.insert_child(&Tk(1), Tk(4), Tv("I am the light in the night"));

        let dfsiter = DfsIter::new(&rt);
        let keys: Vec<Tk> = dfsiter.map(|n| n.0.clone()).collect();
        assert_eq!(keys, &[Tk(0), Tk(2), Tk(1), Tk(3), Tk(5), Tk(4)]);
    }

    #[test]
    fn ancestors() {
        let mut rt: Tree<Tk, Tvm> = Tree::default();
        rt.insert(Tk(0), Tvm(2));
        rt.insert_child(&Tk(0), Tk(2), Tvm(7));
        rt.insert(Tk(1), Tvm(3));
        rt.insert_child(&Tk(1), Tk(3), Tvm(5));
        rt.insert_child(&Tk(3), Tk(5), Tvm(11));
        rt.insert_child(&Tk(1), Tk(4), Tvm(13));

        let ancestors: Vec<(&Tk, &Tvm)> = rt.ancestors(&Tk(5)).collect();
        assert_eq!(ancestors, [(&Tk(5), &Tvm(11)), (&Tk(3), &Tvm(5)), (&Tk(1), &Tvm(3))]);
    }

    #[test]
    fn impl_partial_eq() {
        let mut rt: Tree<Tk, Tvm> = Tree::default();
        rt.insert(Tk(0), Tvm(2));
        rt.insert_child(&Tk(0), Tk(2), Tvm(7));
        rt.insert(Tk(1), Tvm(3));
        rt.insert_child(&Tk(1), Tk(3), Tvm(5));
        rt.insert_child(&Tk(3), Tk(5), Tvm(11));
        rt.insert_child(&Tk(1), Tk(4), Tvm(13));

        let mut rt2: Tree<Tk, Tvm> = Tree::default();
        rt2.insert(Tk(0), Tvm(2));
        rt2.insert_child(&Tk(0), Tk(2), Tvm(7));
        rt2.insert(Tk(1), Tvm(3));
        rt2.insert_child(&Tk(1), Tk(3), Tvm(5));
        rt2.insert_child(&Tk(3), Tk(5), Tvm(11));
        rt2.insert_child(&Tk(1), Tk(4), Tvm(13));

        let mut rt3: Tree<Tk, Tvm> = Tree::default();
        rt3.insert(Tk(0), Tvm(2));
        rt3.insert_child(&Tk(0), Tk(2), Tvm(7));
        rt3.insert(Tk(1), Tvm(3));
        rt3.insert_child(&Tk(1), Tk(3), Tvm(5));
        rt3.insert_child(&Tk(3), Tk(5), Tvm(11));

        assert_eq!(&rt, &rt2);
        assert_ne!(&rt, &rt3);
    }

    #[cfg(feature = "test-slow")]
    #[test]
    fn impl_serde() {
        let mut rt: Tree<Tk, Tvm> = Tree::default();
        rt.insert(Tk(0), Tvm(2));
        rt.insert_child(&Tk(0), Tk(2), Tvm(7));
        rt.insert(Tk(1), Tvm(3));
        rt.insert_child(&Tk(1), Tk(3), Tvm(5));
        rt.insert_child(&Tk(3), Tk(5), Tvm(11));
        rt.insert_child(&Tk(1), Tk(4), Tvm(13));

        assert_tokens(
            &rt,
            &[
                Token::Struct { name: "Tree", len: 3 },
                Token::Str("edges"),
                Token::Map { len: Some(3) },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(0),
                Token::Seq { len: Some(1) },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(2),
                Token::SeqEnd,
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(1),
                Token::Seq { len: Some(2) },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(3),
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(4),
                Token::SeqEnd,
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(3),
                Token::Seq { len: Some(1) },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(5),
                Token::SeqEnd,
                Token::MapEnd,
                Token::Str("nodes"),
                Token::Map { len: Some(6) },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(0),
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(1),
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(2),
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(3),
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(4),
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(5),
                Token::MapEnd,
                Token::Str("roots"),
                Token::Seq { len: Some(2) },
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(0),
                Token::NewtypeStruct { name: "TestKey" },
                Token::U64(1),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        )
    }
}
