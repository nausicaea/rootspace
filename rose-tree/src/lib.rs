use std::{
    collections::{HashMap, VecDeque},
};

// pub struct SgGroup<A, B>(A, B);
//
// impl<A, B> SgGroup<A, B> {
//     pub fn first(&self) -> &A {
//         &self.0
//     }
//
//     pub fn second(&self) -> &B {
//         &self.1
//     }
// }
//
// impl<'a, A, B> std::ops::Mul<&'a SgGroup<A, B>> for &'a SgGroup<A, B>
// where
//     &A: std::ops::Mul<&A, Output =A>,
//     &B: std::ops::Mul<&B, Output =B>,
// {
//     type Output = SgGroup<A, B>;
//
//     fn mul(self, rhs: &'a SgGroup<A, B>) -> Self::Output {
//         SgGroup(&self.0 * &rhs.0, &self.1 * &rhs.1)
//     }
// }
//
// pub struct SceneGraph {
//     world: Hierarchy<Entity, SgGroup<Status, Model>>,
//     ui: Hierarchy<Entity, SgGroup<Status, Model>>,
// }

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "K: Eq + std::hash::Hash + serde::Serialize, V: serde::Serialize", deserialize = "K: Eq + std::hash::Hash + for<'r> serde::Deserialize<'r>, V: for<'r> serde::Deserialize<'r>")))]
#[derive(Debug)]
pub struct Hierarchy<K, V> {
    edges: HashMap<K, Vec<K>>,
    nodes: HashMap<K, V>,
    roots: Vec<K>,
}

impl<K, V> Hierarchy<K, V> {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.roots.clear();
        self.nodes.clear();
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Clone,
{
    pub fn bfs_iter(&self) -> BfsIter<K, V> {
        BfsIter::new(self)
    }

    pub fn dfs_iter(&self) -> DfsIter<K, V> {
        DfsIter::new(self)
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Eq + std::hash::Hash,
{
    pub fn contains(&self, key: &K) -> bool {
        self.nodes.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.nodes.get(key)
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Clone + Eq + std::hash::Hash,
{
    pub fn insert(&mut self, key: K, value: V) -> bool {
        if self.nodes.contains_key(&key) {
            return false;
        }

        self.roots.push(key.clone());
        self.nodes.insert(key, value);

        true
    }

    pub fn insert_child(&mut self, parent: &K, key: K, value: V) -> bool {
        if self.nodes.contains_key(&key) {
            return false;
        }

        self.edges.entry(parent.clone())
            .and_modify(|e| e.push(key.clone()))
            .or_insert_with(|| vec![key.clone()]);

        self.nodes.insert(key, value);

        true
    }

    pub fn remove(&mut self, key: &K) -> bool {
        if !self.nodes.contains_key(key) {
            return false;
        }

        // Push the target node onto the removal queue
        let mut queue: VecDeque<K> = VecDeque::new();
        queue.push_back(key.clone());

        while let Some(k) = queue.pop_front() {
            // Remove all edges of the current node
            queue.extend(self.edges.remove(&k).into_iter().flat_map(|children| children.into_iter()));

            // Remove all dangling edges
            for (_, children) in &mut self.edges {
                children.retain(|c| c != &k);
            }

            // Remove the current node from roots (if it was a tree root)
            self.roots.retain(|r| r != &k);

            // Remove the current node's value
            self.nodes.remove(&k);
        }

        true
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Default,
{
    pub fn traverse_with<F: Fn(Option<&V>, &V) -> V>(&mut self, f: F) {
        let mut queue: VecDeque<(Option<K>, K)> = VecDeque::default();

        queue.extend(self.roots.iter().map(|k| (None, k.clone())));

        while let Some((parent, target)) = queue.pop_front() {
            let new_value = {
                let parent_value = parent.and_then(|p| self.nodes.get(&p));
                let target_value = self.nodes.get(&target)
                    .expect("The node was not found");

                f(parent_value, target_value)
            };

            self.nodes.get_mut(&target)
                .map(|v| *v = new_value)
                .expect("The node was not found");

            queue.extend(self.edges.get(&target).iter().flat_map(|children| children.iter().map(|c| (Some(target.clone()), c.clone()))));
        }
    }
}

impl<K, V> Default for Hierarchy<K, V> {
    fn default() -> Self {
        Hierarchy {
            edges: HashMap::default(),
            nodes: HashMap::default(),
            roots: Vec::default(),
        }
    }
}

impl<K, V> PartialEq for Hierarchy<K, V>
    where
        K: Eq + std::hash::Hash,
        V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.edges.eq(&other.edges) && self.nodes.eq(&other.nodes) && self.roots.eq(&other.roots)
    }
}

pub struct BfsIter<'a, K, V> {
    queue: VecDeque<K>,
    hier: &'a Hierarchy<K, V>,
}

impl<'a, K, V> BfsIter<'a, K, V>
where
    K: Clone,
{
    fn new(hier: &'a Hierarchy<K, V>) -> Self {
        BfsIter {
            queue: hier.roots.iter().cloned().collect(),
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
            self.queue.extend(self.hier.edges.get(&next_node).iter().flat_map(|children| children.iter().cloned()));

            self.hier.nodes.get_key_value(&next_node)
        } else {
            None
        }
    }
}

pub struct DfsIter<'a, K, V> {
    stack: Vec<K>,
    hier: &'a Hierarchy<K, V>,
}

impl<'a, K, V> DfsIter<'a, K, V>
where
    K: Clone,
{
    fn new(hier: &'a Hierarchy<K, V>) -> Self {
        DfsIter {
            stack: hier.roots.iter().rev().cloned().collect(),
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
            self.stack.extend(self.hier.edges.get(&next_node).iter().flat_map(|children| children.iter().rev().cloned()));

            self.hier.nodes.get_key_value(&next_node)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    struct TestKey(usize);

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestValue(&'static str);

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestValueMul(usize);

    impl Default for TestValueMul {
        fn default() -> Self {
            TestValueMul(1)
        }
    }

    impl<'a> Mul<&'a TestValueMul> for &'a TestValueMul {
        type Output = TestValueMul;

        fn mul(self, rhs: &'a TestValueMul) -> Self::Output {
            TestValueMul(self.0 * rhs.0)
        }
    }

    #[test]
    fn impl_default() {
        let _: Hierarchy<TestKey, TestValue> = Default::default();
    }

    #[test]
    fn insert() {
        let mut rt = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("Hello, World!"));
        rt.insert(TestKey(1), TestValue("Good night, World!"));
    }

    #[test]
    fn insert_child() {
        let mut rt = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("Hello, World!"));
        rt.insert_child(&TestKey(0), TestKey(1), TestValue("Good night, World!"));
        rt.insert_child(&TestKey(0), TestKey(2), TestValue("Data for TestKey2"));
    }

    #[test]
    fn no_cycles() {
        let mut rt = Hierarchy::default();
        assert!(rt.insert(TestKey(0), TestValue("0")));
        assert_eq!(rt.get(&TestKey(0)), Some(&TestValue("0")));
        assert!(!rt.insert(TestKey(0), TestValue("Some other value")));
        assert_eq!(rt.get(&TestKey(0)), Some(&TestValue("0")));
        assert!(rt.insert(TestKey(1), TestValue("1")));
        assert_eq!(rt.get(&TestKey(1)), Some(&TestValue("1")));
        assert!(!rt.insert_child(&TestKey(1), TestKey(0), TestValue("Zero")));
        assert_eq!(rt.get(&TestKey(0)), Some(&TestValue("0")));
        assert!(rt.remove(&TestKey(0)));
        assert!(rt.insert_child(&TestKey(1), TestKey(0), TestValue("Zero")));
        assert_eq!(rt.get(&TestKey(0)), Some(&TestValue("Zero")));
    }

    #[test]
    fn get() {
        let mut rt = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("Hello, World!"));
        rt.insert_child(&TestKey(0), TestKey(1), TestValue("Good night, World!"));

        assert_eq!(rt.get(&TestKey(0)), Some(&TestValue("Hello, World!")));
        assert_eq!(rt.get(&TestKey(1)), Some(&TestValue("Good night, World!")));
        assert!(rt.get(&TestKey(2)).is_none());
    }

    #[test]
    fn remove() {
        let mut rt: Hierarchy<TestKey, TestValue> = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("Hello, World!"));
        rt.insert_child(&TestKey(0), TestKey(2), TestValue("Not my tree"));
        rt.insert(TestKey(1), TestValue("Good night, World!"));
        rt.insert_child(&TestKey(1), TestKey(3), TestValue("I am the shadow of the night"));
        rt.insert_child(&TestKey(3), TestKey(5), TestValue("Hissssssss"));
        rt.insert_child(&TestKey(1), TestKey(4), TestValue("I am the light in the night"));

        assert!(rt.remove(&TestKey(1)));
        assert!(rt.contains(&TestKey(0)));
        assert!(rt.contains(&TestKey(2)));
        assert!(!rt.contains(&TestKey(1)));
        assert!(!rt.contains(&TestKey(3)));
        assert!(!rt.contains(&TestKey(4)));
        assert!(!rt.contains(&TestKey(5)));
        assert!(!rt.remove(&TestKey(1)));
    }

    #[test]
    fn is_empty() {
        let mut rt = Hierarchy::default();

        assert!(rt.is_empty());
        rt.insert((), ());
        assert!(!rt.is_empty());
    }

    #[test]
    fn len() {
        let mut rt: Hierarchy<TestKey, TestValue> = Hierarchy::default();

        assert_eq!(rt.len(), 0);
        rt.insert(TestKey(0), TestValue("TestKey(0)"));
        assert_eq!(rt.len(), 1);
        rt.insert_child(&TestKey(0), TestKey(1), TestValue("TestKey(1)"));
        assert_eq!(rt.len(), 2);
    }

    #[test]
    fn contains() {
        let mut rt: Hierarchy<TestKey, TestValue> = Hierarchy::default();

        assert!(!rt.contains(&TestKey(0)));
        rt.insert(TestKey(0), TestValue("TestKey(0)"));
        assert!(rt.contains(&TestKey(0)));
        rt.insert_child(&TestKey(0), TestKey(1), TestValue("TestKey(1)"));
        assert!(rt.contains(&TestKey(1)));
    }

    #[test]
    fn clear() {
        let mut rt: Hierarchy<TestKey, TestValue> = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("TestKey(0)"));
        rt.insert_child(&TestKey(0), TestKey(1), TestValue("TestKey(1)"));

        rt.clear();
        assert!(rt.is_empty());
    }

    #[test]
    fn bfs_iter() {
        let mut rt: Hierarchy<TestKey, TestValue> = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("Hello, World!"));
        rt.insert_child(&TestKey(0), TestKey(2), TestValue("Not my tree"));
        rt.insert(TestKey(1), TestValue("Good night, World!"));
        rt.insert_child(&TestKey(1), TestKey(3), TestValue("I am the shadow of the night"));
        rt.insert_child(&TestKey(3), TestKey(5), TestValue("Hissssssss"));
        rt.insert_child(&TestKey(1), TestKey(4), TestValue("I am the light in the night"));

        let bfsiter = BfsIter::new(&rt);
        let keys: Vec<TestKey> = bfsiter.map(|n| n.0.clone()).collect();
        assert_eq!(
            keys,
            &[TestKey(0), TestKey(1), TestKey(2), TestKey(3), TestKey(4), TestKey(5)]
        );
    }

    #[test]
    fn dfs_iter() {
        let mut rt: Hierarchy<TestKey, TestValue> = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("Hello, World!"));
        rt.insert_child(&TestKey(0), TestKey(2), TestValue("Not my tree"));
        rt.insert(TestKey(1), TestValue("Good night, World!"));
        rt.insert_child(&TestKey(1), TestKey(3), TestValue("I am the shadow of the night"));
        rt.insert_child(&TestKey(3), TestKey(5), TestValue("Hissssssss"));
        rt.insert_child(&TestKey(1), TestKey(4), TestValue("I am the light in the night"));

        let dfsiter = DfsIter::new(&rt);
        let keys: Vec<TestKey> = dfsiter.map(|n| n.0.clone()).collect();
        assert_eq!(
            keys,
            &[TestKey(0), TestKey(2), TestKey(1), TestKey(3), TestKey(5), TestKey(4)]
        );
    }

    #[test]
    fn traverse_with() {
        let mut rt: Hierarchy<TestKey, TestValueMul> = Hierarchy::default();
        rt.insert(TestKey(0), TestValueMul(2));
        rt.insert_child(&TestKey(0), TestKey(2), TestValueMul(7));
        rt.insert(TestKey(1), TestValueMul(3));
        rt.insert_child(&TestKey(1), TestKey(3), TestValueMul(5));
        rt.insert_child(&TestKey(3), TestKey(5), TestValueMul(11));
        rt.insert_child(&TestKey(1), TestKey(4), TestValueMul(13));

        rt.traverse_with(|pn: Option<&TestValueMul>, n: &TestValueMul| pn.map_or(n.clone(), |p| p * n));
        assert_eq!(rt.get(&TestKey(0)), Some(&TestValueMul(2)));
        assert_eq!(rt.get(&TestKey(1)), Some(&TestValueMul(3)));
        assert_eq!(rt.get(&TestKey(2)), Some(&TestValueMul(14)));
        assert_eq!(rt.get(&TestKey(3)), Some(&TestValueMul(15)));
        assert_eq!(rt.get(&TestKey(4)), Some(&TestValueMul(39)));
        assert_eq!(rt.get(&TestKey(5)), Some(&TestValueMul(165)));
    }

    #[test]
    fn impl_partial_eq() {
        let mut rt: Hierarchy<TestKey, TestValueMul> = Hierarchy::default();
        rt.insert(TestKey(0), TestValueMul(2));
        rt.insert_child(&TestKey(0), TestKey(2), TestValueMul(7));
        rt.insert(TestKey(1), TestValueMul(3));
        rt.insert_child(&TestKey(1), TestKey(3), TestValueMul(5));
        rt.insert_child(&TestKey(3), TestKey(5), TestValueMul(11));
        rt.insert_child(&TestKey(1), TestKey(4), TestValueMul(13));

        let mut rt2: Hierarchy<TestKey, TestValueMul> = Hierarchy::default();
        rt2.insert(TestKey(0), TestValueMul(2));
        rt2.insert_child(&TestKey(0), TestKey(2), TestValueMul(7));
        rt2.insert(TestKey(1), TestValueMul(3));
        rt2.insert_child(&TestKey(1), TestKey(3), TestValueMul(5));
        rt2.insert_child(&TestKey(3), TestKey(5), TestValueMul(11));
        rt2.insert_child(&TestKey(1), TestKey(4), TestValueMul(13));

        let mut rt3: Hierarchy<TestKey, TestValueMul> = Hierarchy::default();
        rt3.insert(TestKey(0), TestValueMul(2));
        rt3.insert_child(&TestKey(0), TestKey(2), TestValueMul(7));
        rt3.insert(TestKey(1), TestValueMul(3));
        rt3.insert_child(&TestKey(1), TestKey(3), TestValueMul(5));
        rt3.insert_child(&TestKey(3), TestKey(5), TestValueMul(11));

        assert_eq!(&rt, &rt2);
        assert_ne!(&rt, &rt3);
    }

    #[test]
    fn impl_serde() {

        let mut rt: Hierarchy<TestKey, TestValueMul> = Hierarchy::default();
        rt.insert(TestKey(0), TestValueMul(2));
        rt.insert_child(&TestKey(0), TestKey(2), TestValueMul(7));
        rt.insert(TestKey(1), TestValueMul(3));
        rt.insert_child(&TestKey(1), TestKey(3), TestValueMul(5));
        rt.insert_child(&TestKey(3), TestKey(5), TestValueMul(11));
        rt.insert_child(&TestKey(1), TestKey(4), TestValueMul(13));

        assert_tokens(
            &rt,
            &[
                Token::Struct { name: "Hierarchy", len: 3 },
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
