use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

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
    K: Clone + Eq + std::hash::Hash + PartialEq,
{
    pub fn contains(&self, key: &K) -> bool {
        self.index.contains_key(key)
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
        let (parent_node, target_node) = self.index.get(key)
            .and_then(|it| it.parent.clone().map(|p| (p, it.target.clone())))?;

        // Remove the target node from said parent
        RefCell::borrow_mut(&parent_node).children.retain(|child| &child.borrow().key != key);

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

    fn insert_internal(&mut self, node: &RcNode<K, V>) {
        self.children.push(node.clone());
        self.index.insert(node.borrow().key.clone(), IndexTarget::new(node.clone(), None));
    }

    fn insert_child_internal(&mut self, parent: &K, node: &RcNode<K, V>) {
        let parent_node = self.index.get(parent)
            .map(|it| it.target.clone())
            .expect("Could not find the parent node");
        RefCell::borrow_mut(&parent_node).children.push(node.clone());
        self.index.insert(node.borrow().key.clone(), IndexTarget::new(node.clone(), Some(parent_node.clone())));
    }

    fn rebuild_index(&mut self) {
        self.index.clear();
        for child in &self.children {
            self.index.insert(child.borrow().key.clone(), IndexTarget::new(child.clone(), None));
            RefCell::borrow_mut(child).rebuild_index(&mut self.index, child);
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
            index.insert(child.borrow().key.clone(), IndexTarget::new(child.clone(), Some(parent.clone())));
            child.borrow().rebuild_index(index, &child);
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
        IndexTarget {
            target,
            parent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestKey(usize);

    #[derive(Debug, Clone)]
    struct TestValue(&'static str);

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
    fn update() {
        todo!()
    }

    #[test]
    fn impl_partial_eq() {
        todo!()
    }

    #[test]
    fn impl_serde() {
        todo!()
    }

    #[test]
    fn impl_iterator() {
        todo!()
    }

    #[test]
    fn impl_into_iterator() {
        todo!()
    }
}
