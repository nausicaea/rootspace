use std::ops::{Deref, Mul};

use anyhow::{anyhow, Result};
use ecs::{entity::index::Index, Component, Entity, Resource, SerializationName};
use hierarchy::Hierarchy;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SceneGraph<T>(Hierarchy<Entity, T>);

impl<T> SceneGraph<T>
where
    T: Clone + Default + Component,
    for<'r> &'r T: Mul<&'r T, Output = T>,
{
    pub fn update<D, F>(&mut self, data: D)
    where
        D: Deref<Target = F>,
        F: std::ops::Index<Index, Output = T>,
    {
        self.0
            .update(&|entity, _, parent_datum| Some(parent_datum * &data[entity.into()]))
    }

    pub fn insert(&mut self, entity: Entity) {
        self.0.insert(entity, Default::default())
    }

    pub fn insert_child(&mut self, parent: &Entity, child: Entity) -> Result<()> {
        self.0
            .insert_child(parent, child, Default::default())
            .map_err(|e| anyhow!("Cannot add the entity {} to the parent {}: {}", parent, child, e))
    }

    pub fn remove(&mut self, entity: Entity) {
        let _ = self.0.remove(entity);
    }

    pub fn contains(&self, entity: &Entity) -> bool {
        self.0.iter().any(|(k, _)| k == entity)
    }

    pub fn get(&self, entity: &Entity) -> Option<&T> {
        self.0.iter().filter(|&(k, _)| k == entity).map(|(_, v)| v).last()
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<T> Default for SceneGraph<T> {
    fn default() -> Self {
        SceneGraph(Hierarchy::default())
    }
}

impl<T> Resource for SceneGraph<T> where T: Clone + Default + 'static {}

impl<T> SerializationName for SceneGraph<T> where T: Clone + Default {}

impl<'a, T> IntoIterator for &'a SceneGraph<T>
where
    T: 'a + Clone + Default,
{
    type IntoIter = <&'a Hierarchy<Entity, T> as IntoIterator>::IntoIter;
    type Item = <&'a Hierarchy<Entity, T> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<T> std::fmt::Debug for SceneGraph<T>
where
    T: Clone + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SceneGraph(#nodes: {})", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

    use ecs::{Entities, VecStorage};
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestComponent(usize);

    impl Component for TestComponent {
        type Storage = VecStorage<Self>;
    }

    impl Mul<TestComponent> for TestComponent {
        type Output = TestComponent;

        fn mul(self, rhs: TestComponent) -> TestComponent {
            &self * &rhs
        }
    }

    impl<'a, 'b> Mul<&'a TestComponent> for &'b TestComponent {
        type Output = TestComponent;

        fn mul(self, rhs: &'a TestComponent) -> TestComponent {
            TestComponent(self.0 * rhs.0)
        }
    }

    #[test]
    fn serde() {
        let mut entities = Entities::default();
        let mut sg: SceneGraph<TestComponent> = Default::default();
        sg.insert(entities.create());

        assert_tokens(
            &sg,
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
                Token::Seq { len: Some(2) },
                Token::None,
                Token::Some,
                Token::Tuple { len: 2 },
                Token::Tuple { len: 2 },
                Token::U32(0),
                Token::U32(1),
                Token::TupleEnd,
                Token::NewtypeStruct { name: "TestComponent" },
                Token::U64(0),
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
                Token::Seq { len: Some(1) },
                Token::Some,
                Token::Tuple { len: 3 },
                Token::U32(0),
                Token::U32(1),
                Token::Unit,
                Token::TupleEnd,
                Token::SeqEnd,
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
