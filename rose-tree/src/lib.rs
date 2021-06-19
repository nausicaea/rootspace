use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    rc::Rc,
};

type RcNode<K, V> = Rc<RefCell<Node<K, V>>>;

#[derive(Debug)]
pub struct Hierarchy<K, V> {
    index: HashMap<K, IndexTarget<K, V>>,
    children: Vec<RcNode<K, V>>,
}

impl<K, V> Hierarchy<K, V> {
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn len(&self) -> usize {
        self.index.len()
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
    K: Clone + Eq + std::hash::Hash + PartialEq,
{
    pub fn contains(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.index
            .get(key)
            .map(|it| unsafe { &*(&it.target.borrow().value as *const _) })
    }

    pub fn insert(&mut self, key: K, value: V) {
        let node = Rc::new(RefCell::new(Node::new(key.clone(), value)));
        self.insert_internal(&node);
    }

    pub fn insert_child(&mut self, parent: &K, key: K, value: V) {
        let node = Rc::new(RefCell::new(Node::new(key.clone(), value)));
        self.insert_child_internal(parent, &node);
    }

    pub fn remove(&mut self, key: &K) -> Option<Hierarchy<K, V>> {
        // Retrieve the parent node
        let (parent_node, target_node) = self.index.get(key).map(|it| (it.parent.clone(), it.target.clone()))?;

        if let Some(parent_node) = parent_node {
            // Remove the target node from said parent
            RefCell::borrow_mut(&parent_node)
                .children
                .retain(|child| &child.borrow().key != key);
        } else {
            // Remove the target node from the root node
            self.children.retain(|child| &child.borrow().key != key);
        }

        // Rebuild our own index
        self.rebuild_index();

        // Create a new subtree
        let mut subtree = Hierarchy::<K, V>::default();

        // Insert the new top node
        subtree.insert_internal(&target_node);

        // Rebuild the subtree's index
        subtree.rebuild_index();

        Some(subtree)
    }

    pub fn clear(&mut self) {
        self.children.clear();
        self.index.clear();
    }

    pub fn traverse_with<F: Fn(Option<&V>, &V) -> V>(&mut self, f: F) {
        for child in &self.children {
            let new_value = f(None, &child.borrow().value);
            child.borrow_mut().value = new_value;
            RefCell::borrow(child).traverse_with(&f);
        }
    }

    fn insert_internal(&mut self, node: &RcNode<K, V>) {
        self.children.push(node.clone());
        self.index
            .insert(node.borrow().key.clone(), IndexTarget::new(node.clone(), None));
    }

    fn insert_child_internal(&mut self, parent: &K, node: &RcNode<K, V>) {
        let parent_node = self
            .index
            .get(parent)
            .map(|it| it.target.clone())
            .expect("Could not find the parent node");
        RefCell::borrow_mut(&parent_node).children.push(node.clone());
        self.index.insert(
            node.borrow().key.clone(),
            IndexTarget::new(node.clone(), Some(parent_node.clone())),
        );
    }

    fn rebuild_index(&mut self) {
        self.index.clear();
        for child in &self.children {
            self.index
                .insert(child.borrow().key.clone(), IndexTarget::new(child.clone(), None));
            RefCell::borrow(child).rebuild_index(&mut self.index, child);
        }
    }
}

impl<K, V> Default for Hierarchy<K, V> {
    fn default() -> Self {
        Hierarchy {
            index: HashMap::default(),
            children: Vec::default(),
        }
    }
}

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    children: Vec<RcNode<K, V>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            children: Vec::new(),
        }
    }
}

impl<K, V> Node<K, V>
where
    K: Clone + Eq + std::hash::Hash + PartialEq,
{
    fn rebuild_index(&self, index: &mut HashMap<K, IndexTarget<K, V>>, parent: &RcNode<K, V>) {
        for child in &self.children {
            index.insert(
                child.borrow().key.clone(),
                IndexTarget::new(child.clone(), Some(parent.clone())),
            );
            child.borrow().rebuild_index(index, &child);
        }
    }

    fn traverse_with<F: Fn(Option<&V>, &V) -> V>(&self, f: &F) {
        for child in &self.children {
            let new_value = f(Some(&self.value), &child.borrow().value);
            child.borrow_mut().value = new_value;
            RefCell::borrow(child).traverse_with(f);
        }
    }
}

#[derive(Debug)]
struct IndexTarget<K, V> {
    target: RcNode<K, V>,
    parent: Option<RcNode<K, V>>,
}

impl<K, V> IndexTarget<K, V> {
    fn new(target: RcNode<K, V>, parent: Option<RcNode<K, V>>) -> Self {
        IndexTarget { target, parent }
    }
}

pub struct BfsIter<'a, K, V> {
    queue: VecDeque<RcNode<K, V>>,
    _h: PhantomData<&'a Hierarchy<K, V>>,
}

impl<'a, K: Clone, V> BfsIter<'a, K, V> {
    fn new(hier: &'a Hierarchy<K, V>) -> Self {
        BfsIter {
            queue: hier.children.iter().cloned().collect(),
            _h: PhantomData::default(),
        }
    }
}

impl<'a, K, V> Iterator for BfsIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.queue.pop_front() {
            self.queue.extend(next_node.borrow().children.iter().cloned());

            unsafe {
                let bnn: Ref<Node<K, V>> = next_node.borrow();
                Some((&*(&bnn.key as *const _), &*(&bnn.value as *const _)))
            }
        } else {
            None
        }
    }
}

pub struct DfsIter<'a, K, V> {
    stack: Vec<RcNode<K, V>>,
    _h: PhantomData<&'a Hierarchy<K, V>>,
}

impl<'a, K, V> DfsIter<'a, K, V> {
    fn new(hier: &'a Hierarchy<K, V>) -> Self {
        DfsIter {
            stack: hier.children.iter().rev().cloned().collect(),
            _h: PhantomData::default(),
        }
    }
}

impl<'a, K, V> Iterator for DfsIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.stack.pop() {
            self.stack.extend(next_node.borrow().children.iter().rev().cloned());

            unsafe {
                let bnn: Ref<Node<K, V>> = next_node.borrow();
                Some((&*(&bnn.key as *const _), &*(&bnn.value as *const _)))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestKey(usize);

    #[derive(Debug, Clone, PartialEq)]
    struct TestValue(&'static str);

    #[derive(Debug, Clone, PartialEq)]
    struct TestValueMul(usize);

    impl<'a> Mul<&'a TestValueMul> for &'a TestValueMul {
        type Output = TestValueMul;

        fn mul(self, rhs: &'a TestValueMul) -> Self::Output {
            TestValueMul(self.0 * rhs.0)
        }
    }

    #[test]
    fn internal_representation() {
        let mut rt: Hierarchy<TestKey, TestValue> = Hierarchy::default();
        rt.insert(TestKey(0), TestValue("Hello, World!"));
        rt.insert_child(&TestKey(0), TestKey(2), TestValue("Not my tree"));
        rt.insert(TestKey(1), TestValue("Good night, World!"));
        rt.insert_child(&TestKey(1), TestKey(3), TestValue("I am the shadow of the night"));
        rt.insert_child(&TestKey(3), TestKey(5), TestValue("Hissssssss"));
        rt.insert_child(&TestKey(1), TestKey(4), TestValue("I am the light in the night"));

        assert_eq!(rt.index.len(), 6);
        assert_eq!(rt.children.len(), 2);
        assert_eq!(rt.children[0].borrow().children.len(), 1);
        assert_eq!(rt.children[1].borrow().children.len(), 2);
        assert_eq!(rt.children[1].borrow().children[0].borrow().children.len(), 1);
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

        let subtree1: Hierarchy<TestKey, TestValue> = rt.remove(&TestKey(1)).expect("Expected a valid subtree");
        assert!(!subtree1.contains(&TestKey(0)));
        assert!(subtree1.contains(&TestKey(1)));
        assert!(!subtree1.contains(&TestKey(2)));
        assert!(subtree1.contains(&TestKey(3)));
        assert!(subtree1.contains(&TestKey(4)));
        assert!(subtree1.contains(&TestKey(5)));
        assert!(rt.contains(&TestKey(0)));
        assert!(!rt.contains(&TestKey(1)));
        assert!(rt.contains(&TestKey(2)));
        assert!(!rt.contains(&TestKey(3)));
        assert!(!rt.contains(&TestKey(4)));
        assert!(!rt.contains(&TestKey(5)));
        let subtree2: Option<Hierarchy<TestKey, TestValue>> = rt.remove(&TestKey(1));
        assert!(subtree2.is_none());
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
        todo!()
    }

    #[test]
    fn impl_serde() {
        todo!()
    }
}
