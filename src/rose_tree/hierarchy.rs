#![allow(dead_code)]

use std::{
    collections::VecDeque,
    fmt::{Debug, Display, Formatter},
    hash::Hash,
};

use serde::{Deserialize, Serialize};

use crate::{ecs::resource::Resource, rose_tree::tree::Tree};

#[derive(Serialize, Deserialize)]
#[serde(
    transparent,
    bound(
        serialize = "K: Ord + std::hash::Hash + serde::Serialize",
        deserialize = "K: Ord + std::hash::Hash + for<'r> serde::Deserialize<'r>"
    )
)]
#[derive(Debug, Clone)]
pub struct Hierarchy<K>(Tree<K, ()>);

impl<K> Hierarchy<K> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl<K> Hierarchy<K>
where
    K: Eq + Hash,
{
    pub fn has_children<J: AsRef<K>>(&self, key: J) -> bool {
        self.0.has_children(key)
    }
}

impl<K> Hierarchy<K>
where
    K: Clone,
{
    pub fn bfs_iter(&self) -> BfsIter<K> {
        BfsIter::new(self)
    }

    pub fn dfs_iter(&self) -> DfsIter<K> {
        DfsIter::new(self)
    }

    pub fn ancestors<J: AsRef<K>>(&self, key: J) -> AncestorsIter<K> {
        AncestorsIter::new(self, key.as_ref())
    }
}

impl<K> Hierarchy<K>
where
    K: Eq + Hash,
{
    pub fn contains_key<J: AsRef<K>>(&self, key: J) -> bool {
        self.0.contains_key(key)
    }
}

impl<K> Hierarchy<K>
where
    K: Clone + Ord + Hash,
{
    pub fn insert<I: Into<K>>(&mut self, key: I) -> bool {
        self.0.insert(key, ())
    }

    pub fn insert_child<J: AsRef<K>, I: Into<K>>(&mut self, parent: J, key: I) -> bool {
        self.0.insert_child(parent, key, ())
    }

    pub fn remove<J: AsRef<K>>(&mut self, key: J) -> bool {
        self.0.remove(key)
    }
}

impl<K> Default for Hierarchy<K>
where
    K: Ord,
{
    fn default() -> Self {
        Hierarchy(Tree::default())
    }
}

impl<K> Display for Hierarchy<K>
where
    K: Display + Eq + PartialEq + Hash,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<K> PartialEq for Hierarchy<K>
where
    K: Eq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K> Resource for Hierarchy<K> where K: 'static + Send + Sync {}

impl<D, K: Ord> crate::ecs::with_dependencies::WithDependencies<D> for Hierarchy<K> {
    #[tracing::instrument(skip_all)]
    async fn with_deps(_: &D) -> Result<Self, anyhow::Error> {
        Ok(Hierarchy::default())
    }
}

pub struct AncestorsIter<'a, K> {
    key: Option<K>,
    hier: &'a Hierarchy<K>,
}

impl<'a, K> AncestorsIter<'a, K>
where
    K: Clone,
{
    fn new(hier: &'a Hierarchy<K>, key: &K) -> Self {
        AncestorsIter {
            key: Some(key.clone()),
            hier,
        }
    }
}

impl<'a, K> Iterator for AncestorsIter<'a, K>
where
    K: Clone + Ord + Eq + Hash,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.key.clone()?;

        if !self.hier.0.nodes.contains_key(&key) {
            self.key = None;
            return None;
        }

        self.key = self.hier.0.parents.get(&key).and_then(|p| p.clone());
        Some(key)
    }
}

pub struct BfsIter<'a, K> {
    queue: VecDeque<K>,
    hier: &'a Hierarchy<K>,
}

impl<'a, K> BfsIter<'a, K>
where
    K: Clone,
{
    fn new(hier: &'a Hierarchy<K>) -> Self {
        BfsIter {
            queue: hier
                .0
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

impl<'a, K> Iterator for BfsIter<'a, K>
where
    K: Clone + Eq + Hash,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.queue.pop_front() {
            self.queue.extend(
                self.hier
                    .0
                    .edges
                    .get(&next_node)
                    .iter()
                    .flat_map(|children| children.iter().cloned()),
            );

            Some(next_node)
        } else {
            None
        }
    }
}

pub struct DfsIter<'a, K> {
    stack: Vec<K>,
    hier: &'a Hierarchy<K>,
}

impl<'a, K> DfsIter<'a, K>
where
    K: Clone,
{
    fn new(hier: &'a Hierarchy<K>) -> Self {
        DfsIter {
            stack: hier
                .0
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

impl<'a, K> Iterator for DfsIter<'a, K>
where
    K: Clone + Eq + Hash,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.stack.pop() {
            self.stack.extend(
                self.hier
                    .0
                    .edges
                    .get(&next_node)
                    .iter()
                    .flat_map(|children| children.iter().rev().cloned()),
            );

            Some(next_node)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{iter::Product, ops::Mul};

    use super::*;
    use crate::{
        ecs::{
            component::Component,
            entities::Entities,
            entity::index::Index,
            registry::{End, ResourceRegistry},
            storage::{vec_storage::VecStorage, Storage},
            world::World,
        },
        Reg,
    };

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    struct Tk(usize);

    impl AsRef<Tk> for Tk {
        fn as_ref(&self) -> &Tk {
            self
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct Tc(usize);

    impl Component for Tc {
        type Storage = VecStorage<Self>;
    }

    impl Mul<Tc> for Tc {
        type Output = Tc;

        fn mul(self, rhs: Tc) -> Tc {
            &self * &rhs
        }
    }

    impl<'a, 'b> Mul<&'a Tc> for &'b Tc {
        type Output = Tc;

        fn mul(self, rhs: &'a Tc) -> Tc {
            Tc(self.0 * rhs.0)
        }
    }

    impl<'a> Product<&'a Tc> for Tc {
        fn product<I: Iterator<Item = &'a Tc>>(iter: I) -> Self {
            iter.fold(Tc(1), |state, value| &state * value)
        }
    }

    impl Product for Tc {
        fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Tc(1), |state, value| state * value)
        }
    }

    #[test]
    fn hierarchy_reg_macro() {
        type _RR = Reg![Hierarchy<Tk>];
    }

    #[test]
    fn hierarchy_resource_registry() {
        let _rr = ResourceRegistry::push(End, Hierarchy::<Tk>::default());
    }

    #[tokio::test]
    async fn hierarchy_world() {
        let _w = World::with_dependencies::<Reg![Hierarchy<Tk>], Reg![], Reg![], (), Reg![], _>(&())
            .await
            .unwrap();
    }

    #[test]
    fn impl_default() {
        let _: Hierarchy<Tk> = Default::default();
    }

    #[test]
    fn insert() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert(Tk(1));
    }

    #[test]
    fn insert_child() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert_child(&Tk(0), Tk(1));
        rt.insert_child(&Tk(0), Tk(2));
    }

    #[test]
    fn no_cycles() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        assert!(rt.insert(Tk(0)));
        assert!(!rt.insert(Tk(0)));
        assert!(rt.insert(Tk(1)));
        assert!(!rt.insert_child(&Tk(1), Tk(0)));
        assert!(rt.remove(&Tk(0)));
        assert!(rt.insert_child(&Tk(1), Tk(0)));
    }

    #[test]
    fn remove() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert_child(&Tk(0), Tk(2));
        rt.insert(Tk(1));
        rt.insert_child(&Tk(1), Tk(3));
        rt.insert_child(&Tk(3), Tk(5));
        rt.insert_child(&Tk(1), Tk(4));

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
    fn has_children() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert(Tk(1));
        rt.insert_child(&Tk(1), Tk(3));
        rt.insert_child(&Tk(3), Tk(5));
        rt.insert_child(&Tk(1), Tk(4));

        assert!(!rt.has_children(&Tk(0)));
        assert!(rt.has_children(&Tk(1)));
    }

    #[test]
    fn is_empty() {
        let mut rt: Hierarchy<()> = Hierarchy::default();

        assert!(rt.is_empty());
        rt.insert(());
        assert!(!rt.is_empty());
    }

    #[test]
    fn len() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();

        assert_eq!(rt.len(), 0);
        rt.insert(Tk(0));
        assert_eq!(rt.len(), 1);
        rt.insert_child(&Tk(0), Tk(1));
        assert_eq!(rt.len(), 2);
    }

    #[test]
    fn contains() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();

        assert!(!rt.contains_key(&Tk(0)));
        rt.insert(Tk(0));
        assert!(rt.contains_key(&Tk(0)));
        rt.insert_child(&Tk(0), Tk(1));
        assert!(rt.contains_key(&Tk(1)));
    }

    #[test]
    fn clear() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert_child(&Tk(0), Tk(1));

        rt.clear();
        assert!(rt.is_empty());
    }

    #[test]
    fn bfs_iter() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert_child(&Tk(0), Tk(2));
        rt.insert(Tk(1));
        rt.insert_child(&Tk(1), Tk(3));
        rt.insert_child(&Tk(3), Tk(5));
        rt.insert_child(&Tk(1), Tk(4));

        let bfsiter = BfsIter::new(&rt);
        let keys: Vec<Tk> = bfsiter.collect();
        assert_eq!(keys, &[Tk(0), Tk(1), Tk(2), Tk(3), Tk(4), Tk(5)]);
    }

    #[test]
    fn dfs_iter() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert_child(&Tk(0), Tk(2));
        rt.insert(Tk(1));
        rt.insert_child(&Tk(1), Tk(3));
        rt.insert_child(&Tk(3), Tk(5));
        rt.insert_child(&Tk(1), Tk(4));

        let dfsiter = DfsIter::new(&rt);
        let keys: Vec<Tk> = dfsiter.collect();
        assert_eq!(keys, &[Tk(0), Tk(2), Tk(1), Tk(3), Tk(5), Tk(4)]);
    }

    #[test]
    fn ancestors() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert_child(&Tk(0), Tk(2));
        rt.insert(Tk(1));
        rt.insert_child(&Tk(1), Tk(3));
        rt.insert_child(&Tk(3), Tk(5));
        rt.insert_child(&Tk(1), Tk(4));

        let ancestors: Vec<Tk> = rt.ancestors(&Tk(5)).collect();
        assert_eq!(ancestors, [Tk(5), Tk(3), Tk(1)]);
    }

    #[test]
    fn ancestor_and_ecs_usage() {
        let mut hierarchy = Entities::default();
        let mut s = <Tc as Component>::Storage::default();
        let mut rt: Hierarchy<Index> = Hierarchy::default();

        let e1 = hierarchy.create();
        s.insert(e1, Tc(2));
        rt.insert(e1);
        let e2 = hierarchy.create();
        s.insert(e2, Tc(3));
        rt.insert(e2);
        let e3 = hierarchy.create();
        s.insert(e3, Tc(5));
        rt.insert_child(e1, e3);
        let e4 = hierarchy.create();
        s.insert(e4, Tc(7));
        rt.insert_child(e3, e4);

        assert_eq!(rt.ancestors(e4).filter_map(|idx| s.get(idx)).product::<Tc>(), Tc(70));
    }

    #[test]
    fn impl_partial_eq() {
        let mut rt: Hierarchy<Tk> = Hierarchy::default();
        rt.insert(Tk(0));
        rt.insert_child(&Tk(0), Tk(2));
        rt.insert(Tk(1));
        rt.insert_child(&Tk(1), Tk(3));
        rt.insert_child(&Tk(3), Tk(5));
        rt.insert_child(&Tk(1), Tk(4));

        let mut rt2: Hierarchy<Tk> = Hierarchy::default();
        rt2.insert(Tk(0));
        rt2.insert_child(&Tk(0), Tk(2));
        rt2.insert(Tk(1));
        rt2.insert_child(&Tk(1), Tk(3));
        rt2.insert_child(&Tk(3), Tk(5));
        rt2.insert_child(&Tk(1), Tk(4));

        let mut rt3: Hierarchy<Tk> = Hierarchy::default();
        rt3.insert(Tk(0));
        rt3.insert_child(&Tk(0), Tk(2));
        rt3.insert(Tk(1));
        rt3.insert_child(&Tk(1), Tk(3));
        rt3.insert_child(&Tk(3), Tk(5));

        assert_eq!(&rt, &rt2);
        assert_ne!(&rt, &rt3);
    }
}
