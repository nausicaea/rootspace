use nalgebra::Vector3;
use std::collections::{HashMap, BTreeMap};

#[derive(Debug)]
pub struct Octree<K> {
    edges: HashMap<NodeIndex, [Option<NodeIndex>; 8]>,
    parents: BTreeMap<NodeIndex, Option<NodeIndex>>,
    nodes: HashMap<NodeIndex, Node<K>>,
    keys: HashMap<K, NodeIndex>,
    root: NodeIndex,
    max_keys_per_node: usize,
    max_node_index: NodeIndex,
    free_node_index: Vec<NodeIndex>,
}

impl<K> Octree<K> {
    pub fn new<V: Into<Vector3<f32>>>(origin: V, world_size: f32, max_keys_per_node: usize) -> Self {
        let mut free_node_index = Vec::default();
        let mut max_node_index = NodeIndex::default();
        let root = free_node_index.pop().unwrap_or_else(|| max_node_index.post_increment());

        let mut nodes = HashMap::default();
        nodes.insert(root, Node {
            center: origin.into(),
            size: world_size,
            data: Vec::default(),
        });

        let mut edges = HashMap::default();
        edges.insert(root, [None; 8]);

        let mut parents = BTreeMap::default();
        parents.insert(root, None);

        Octree {
            edges,
            parents,
            nodes,
            keys: HashMap::default(),
            root,
            max_keys_per_node,
            max_node_index,
            free_node_index,
        }
    }
}

impl<K> Octree<K>
where
    K: Eq + std::hash::Hash + Clone,
{
    pub fn find_neighbours<V: Into<Vector3<f32>>>(&self, position: V) -> Vec<&K> {
        let position = position.into();

        self.find_neighbours_internal(Some(self.root), position)
    }

    pub fn insert<I: Into<K>, V: Into<Vector3<f32>>>(&mut self, key: I, position: V) -> bool {
        let key = key.into();
        let position = position.into();

        // Exit early if the node key is already contained within the tree
        if self.keys.contains_key(&key) {
            return false;
        }

        self.insert_internal(None, Some(self.root), key, position)
    }

    pub fn remove<J: AsRef<K>>(&mut self, key: J) -> bool {
        let key = key.as_ref();

        if let Some(&node) = self.keys.get(key) {
            let empty_node = self.nodes.get_mut(&node)
                .map(|n| {
                    n.data.retain(|d| &d.key != key);
                    n.data.is_empty()
                })
                .expect("Could not find the corresponding node");
            self.keys.remove(key);

            if empty_node {
                let parent = match self.parents.get(&node) {
                    Some(Some(p)) => *p,
                    _ => return true,
                };
                self.nodes.remove(&node);
                self.edges.remove(&node);
                self.edges.get_mut(&parent)
                    .map(|e| for edge in e {
                        if edge == &Some(node) {
                            *edge = None;
                        }
                    })
                    .expect("Could not find the node parent's edges");
                self.parents.remove(&node);
                self.keys.retain(|_, n| n != &node);
                self.free_node_index.push(node);
            }

            return true;
        }

        false
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn contains_key<J: AsRef<K>>(&self, key: J) -> bool {
        self.keys.contains_key(key.as_ref())
    }

    pub fn clear(&mut self) {
        let mut root_node = self.nodes.remove(&self.root)
            .expect("Could not find the root node's data");
        root_node.data.clear();

        self.nodes.clear();
        self.keys.clear();
        self.parents.clear();
        self.edges.clear();
        self.free_node_index.clear();
        self.max_node_index = NodeIndex::default();

        self.root = self.max_node_index.post_increment();
        self.nodes.insert(self.root, root_node);
        self.edges.insert(self.root, [None; 8]);
        self.parents.insert(self.root, None);
    }

    fn find_neighbours_internal(&self, node: Option<NodeIndex>, position: Vector3<f32>) -> Vec<&K> {
        if let Some(node) = node {
            let edges = self.edges.get(&node).expect("Could not find the edges for the current node");

            if edges.iter().all(|e| e.is_none()) {
                return self.nodes.get(&node).map(|n| n.data.iter().map(|d| &d.key).collect()).expect("Could not find the node's data");
            }

            let branch_id = self.find_branch(node, position);
            let child = edges[branch_id];
            self.find_neighbours_internal(child, position)
        } else {
            Vec::default()
        }
    }

    fn insert_internal(&mut self, parent: Option<NodeIndex>, node: Option<NodeIndex>, key: K, position: Vector3<f32>) -> bool {
        // Was the method called with a current node?
        // Yes:
        //      Does the current node have edges?
        //      Yes:
        //           Determine the correct branch and descend one recursion level
        //      No:
        //           Is the current node full?
        //           Yes:
        //               Add the new key and the current node's keys to a queue, and subdivide each key into new child nodes,
        //               emptying the current node's data. For each key, descend one recursion level.
        //               Return true
        //           No:
        //               Insert the key into the current node (modifies only self.nodes[&node].data and self.keys)
        //               Return true
        // No:
        //      Create a new node as child of the parent, and insert the data there
        //      Return true
        if let Some(node) = node {
            let edges = self.edges.get(&node).expect("Cound not find the node's edges");
            if edges.iter().any(|e| e.is_some()) {
                let branch_id = self.find_branch(node, position);
                let child_node_index = edges[branch_id];
                self.insert_internal(Some(node), child_node_index, key, position)
            } else {
                let node_data_len = self.nodes.get(&node).map(|n| n.data.len()).expect("Could not find the node's data");
                if node_data_len >= self.max_keys_per_node {
                    let data_drain: Vec<_> = self.nodes.get_mut(&node)
                        .map(|n| n.data.drain(..).chain(std::iter::once(NodeData { key, position })))
                        .expect("Could not find the node's data")
                        .collect();
                    for datum in data_drain {
                        let branch_id = self.find_branch(node, datum.position);
                        let edges = self.edges.get(&node).expect("Cound not find the node's edges");
                        let child_node_index = edges[branch_id];
                        self.insert_internal(Some(node), child_node_index, datum.key, datum.position);
                    }
                    true
                } else {
                    self.nodes.get_mut(&node)
                        .map(|n| n.data.push(NodeData { key: key.clone(), position }))
                        .expect("Could not find the node's data");
                    self.keys.insert(key, node);
                    true
                }
            }
        } else {
            let parent = parent.expect("Expected a parent node index");
            let (parent_center, parent_size) = self.nodes.get(&parent)
                .map(|n| (n.center, n.size))
                .expect("Could not find the parent node's data");
            self.add_node(parent, parent_center, parent_size, key, position);
            true
        }
    }

    fn add_node(&mut self, parent: NodeIndex, parent_center: Vector3<f32>, parent_size: f32, key: K, position: Vector3<f32>) -> NodeIndex {
        let branch_id = self.find_branch(parent, position);
        let node_index = self.free_node_index.pop().unwrap_or_else(|| self.max_node_index.post_increment());
        self.nodes.insert(node_index, Node {
            center: Self::calculate_branch_center(parent_center, parent_size, branch_id),
            size: parent_size / 2.0,
            data: vec![NodeData { key: key.clone(), position }],
        });
        self.edges.get_mut(&parent)
            .map(|e| e[branch_id] = Some(node_index))
            .expect("Could not find the parent's edges");
        self.edges.insert(node_index, [None; 8]);
        self.parents.insert(node_index, Some(parent));
        self.keys.insert(key, node_index);
        node_index
    }

    fn calculate_branch_center(parent_center: Vector3<f32>, parent_size: f32, branch_id: usize) -> Vector3<f32> {
        let offset = parent_size / 2.0;
        match branch_id {
            0 => Vector3::new(parent_center.x + offset, parent_center.y + offset, parent_center.z + offset),
            1 => Vector3::new(parent_center.x + offset, parent_center.y + offset, parent_center.z - offset),
            2 => Vector3::new(parent_center.x + offset, parent_center.y - offset, parent_center.z + offset),
            3 => Vector3::new(parent_center.x + offset, parent_center.y - offset, parent_center.z - offset),
            4 => Vector3::new(parent_center.x - offset, parent_center.y + offset, parent_center.z + offset),
            5 => Vector3::new(parent_center.x - offset, parent_center.y + offset, parent_center.z - offset),
            6 => Vector3::new(parent_center.x - offset, parent_center.y - offset, parent_center.z + offset),
            7 => Vector3::new(parent_center.x - offset, parent_center.y - offset, parent_center.z - offset),
            _ => panic!("Unsupported branch number {}", branch_id),
        }
    }

    fn find_branch(&self, node: NodeIndex, p: Vector3<f32>) -> usize {
        let np = self.nodes.get(&node)
            .map(|n| n.center)
            .expect("Could not find the node's data");

        if p.x >= np.x && p.y >= np.y && p.z >= np.z {
            return 0;
        } else if p.x >= np.x && p.y >= np.y && p.z < np.z {
            return 1;
        } else if p.x >= np.x && p.y < np.y && p.z >= np.z {
            return 2;
        } else if p.x >= np.x && p.y < np.y && p.z < np.z {
            return 3;
        } else if p.x < np.x && p.y >= np.y && p.z >= np.z {
            return 4;
        } else if p.x < np.x && p.y >= np.y && p.z < np.z {
            return 5;
        } else if p.x < np.x && p.y < np.y && p.z >= np.z {
            return 6;
        } else if p.x < np.x && p.y < np.y && p.z < np.z {
            return 7;
        } else {
            unimplemented!()
        }
    }
}

impl<K> Default for Octree<K> {
    fn default() -> Self {
        Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 10)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NodeIndex(usize);

impl NodeIndex {
    fn post_increment(&mut self) -> NodeIndex {
        let tmp = *self;
        self.0 += 1;
        tmp
    }
}

impl Default for NodeIndex {
    fn default() -> Self {
        NodeIndex(0)
    }
}

#[derive(Debug, Clone)]
struct Node<K> {
    center: Vector3<f32>,
    size: f32,
    data: Vec<NodeData<K>>,
}

#[derive(Debug, Clone)]
struct NodeData<K> {
    key: K,
    position: Vector3<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
    struct Tk(usize);

    impl AsRef<Tk> for Tk {
        fn as_ref(&self) -> &Tk {
            self
        }
    }

    #[test]
    fn impl_default() {
        let ot: Octree<Tk> = Default::default();
        assert_eq!(ot.max_keys_per_node, 10);
        assert_eq!(ot.nodes[&ot.root].center, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(ot.nodes[&ot.root].size, 15000.0);
    }

    #[test]
    fn new() {
        let _: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 5);
    }

    #[test]
    fn insert() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);
        assert!(ot.insert(Tk(0), Vector3::new(0.0, 1.0, 0.0)));
        assert!(!ot.insert(Tk(0), Vector3::new(1.0, 1.0, 1.0)));
        assert!(ot.insert(Tk(1), Vector3::new(0.0, 1.0, 1.0)));
        assert!(ot.insert(Tk(2), Vector3::new(1.0, 0.0, 1.0)));
    }

    #[test]
    fn remove() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);
        ot.insert(Tk(0), Vector3::new(0.0, 1.0, 0.0));
        ot.insert(Tk(1), Vector3::new(0.0, 1.0, 1.0));
        ot.insert(Tk(2), Vector3::new(1.0, 0.0, 1.0));

        assert!(!ot.remove(&Tk(3)));
        assert!(ot.remove(&Tk(0)));
        assert!(ot.remove(&Tk(2)));
        assert!(ot.remove(&Tk(1)));
    }

    #[test]
    fn is_empty() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);

        assert!(ot.is_empty());

        ot.insert(Tk(0), Vector3::new(0.0, 1.0, 0.0));
        assert!(!ot.is_empty());
        ot.remove(&Tk(0));
        assert!(ot.is_empty());
    }

    #[test]
    fn len() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);

        assert_eq!(ot.len(), 0);

        ot.insert(Tk(0), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(ot.len(), 1);
        ot.insert(Tk(1), Vector3::new(0.0, 1.0, 1.0));
        assert_eq!(ot.len(), 2);
        ot.remove(&Tk(0));
        assert_eq!(ot.len(), 1);
    }

    #[test]
    fn contains_key() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);

        assert!(!ot.contains_key(&Tk(0)));
        ot.insert(Tk(0), Vector3::new(0.0, 1.0, 0.0));
        assert!(ot.contains_key(&Tk(0)));
        ot.remove(&Tk(0));
        assert!(!ot.contains_key(&Tk(0)));
    }

    #[test]
    fn clear() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);
        ot.insert(Tk(0), Vector3::new(0.0, 1.0, 0.0));
        ot.insert(Tk(1), Vector3::new(0.0, 1.0, 1.0));
        ot.insert(Tk(2), Vector3::new(1.0, 0.0, 1.0));

        ot.clear();
        assert!(ot.is_empty());
    }

    #[test]
    fn num_nodes() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);
        assert_eq!(ot.num_nodes(), 1);

        ot.insert(Tk(0), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(ot.num_nodes(), 1);

        ot.insert(Tk(1), Vector3::new(0.0, 1.0, 1.0));
        assert_eq!(ot.num_nodes(), 1);

        ot.insert(Tk(2), Vector3::new(1.0, 0.0, 1.0));
        assert_eq!(ot.num_nodes(), 18);

        ot.clear();
        assert_eq!(ot.num_nodes(), 1);

        ot.insert(Tk(0), Vector3::new(0.0, 1.0, 1.0));
        ot.insert(Tk(1), Vector3::new(-1.0, -1.0, 0.0));
        ot.insert(Tk(2), Vector3::new(1.0, 1.0, -1.0));
        assert_eq!(ot.num_nodes(), 4);
    }

    #[test]
    fn find_neighbours() {
        let mut ot: Octree<Tk> = Octree::new(Vector3::new(0.0, 0.0, 0.0), 15000.0, 2);
        ot.insert(Tk(0), Vector3::new(0.5, 0.5, 0.5));
        ot.insert(Tk(1), Vector3::new(0.5, 0.5, -0.5));
        ot.insert(Tk(2), Vector3::new(0.5, -0.5, 0.5));
        ot.insert(Tk(3), Vector3::new(0.5, -0.5, -0.5));
        ot.insert(Tk(4), Vector3::new(-0.5, 0.5, 0.5));
        ot.insert(Tk(5), Vector3::new(-0.5, 0.5, -0.5));
        ot.insert(Tk(6), Vector3::new(-0.5, -0.5, 0.5));
        ot.insert(Tk(7), Vector3::new(-0.5, -0.5, -0.5));

        assert_eq!(ot.len(), 8);
        assert_eq!(ot.num_nodes(), 9);
        assert_eq!(ot.find_neighbours(Vector3::new(0.75, 0.75, 0.75)), vec![&Tk(0)]);
        assert_eq!(ot.find_neighbours(Vector3::new(-0.75, 0.75, 0.75)), vec![&Tk(4)]);
        assert_eq!(ot.find_neighbours(Vector3::new(0.0, 0.0, 0.0)), vec![&Tk(0)]);
    }
}