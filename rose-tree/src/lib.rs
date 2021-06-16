use std::cell::RefCell;

#[derive(Debug)]
pub struct Hierarchy<K, V> {
    children: Vec<RefCell<Node<K, V>>>,
}

impl<K, V> Hierarchy<K, V> {
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.children.push(RefCell::new(Node::new(key, value)))
    }
}

impl<K, V> Hierarchy<K, V>
where
    K: PartialEq,
{
    pub fn contains(&self, key: &K) -> bool {
        todo!()
    }

    pub fn insert_child(&mut self, parent: &K, key: K, value: V) {
        todo!()
    }

    pub fn remove(&mut self, key: &K) -> Option<Hierarchy<K, V>> {
        todo!()
    }
}

impl<K, V> Default for Hierarchy<K, V> {
    fn default() -> Self {
        Hierarchy {
            children: Vec::default(),
        }
    }
}

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    children: Vec<RefCell<Node<K, V>>>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestKey(usize);

    #[derive(Debug, Clone)]
    struct TestValue(&'static str);

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
        todo!()
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

        let subtree1: Hierarchy<TestKey, TestValue> = rt.remove(&TestKey(1)).unwrap();
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
        let mut rt = Hierarchy::<(), ()>::default();

        assert_eq!(rt.len(), 0);
        rt.insert((), ());
        assert_eq!(rt.len(), 1);
    }

    #[test]
    fn contains() {
        todo!()
    }

    #[test]
    fn find() {
        todo!()
    }

    #[test]
    fn clear() {
        todo!()
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
