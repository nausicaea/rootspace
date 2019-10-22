use ecs::{Component, Entity, Resource, Storage};
use hierarchy::{Hierarchy, RawNodes};
use serde::{Deserialize, Serialize};
use std::{fmt, ops::Mul};
use typename::TypeName;

#[derive(Default, Clone, PartialEq, TypeName, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SceneGraph<T>(Hierarchy<Entity, T>)
where
    T: Clone + Default;

impl<T> SceneGraph<T>
where
    T: Clone + Default + Component,
    for<'r> &'r T: Mul<&'r T, Output = T>,
{
    pub fn update(&mut self, data: &<T as Component>::Storage) {
        self.0
            .update(&|entity, _, parent_datum| data.get(entity).map(|current_datum| parent_datum * current_datum))
    }

    pub fn insert(&mut self, entity: Entity) {
        self.0.insert(entity, Default::default())
    }

    pub fn get(&self, entity: &Entity) -> Option<&T> {
        self.0.iter().filter(|&(k, _)| k == entity).map(|(_, v)| v).last()
    }

    pub fn iter(&self) -> RawNodes<Entity, T> {
        self.0.iter()
    }
}

impl<T> Resource for SceneGraph<T> where T: Clone + Default + 'static {}

impl<T> fmt::Debug for SceneGraph<T>
where
    T: Clone + Default,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SceneGraph(#nodes: {})", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::{Entities, VecStorage};
    use serde_test::{assert_tokens, Token};
    use std::ops::Mul;

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
                Token::NewtypeStruct { name: "HierNode" },
                Token::None,
                Token::NewtypeStruct { name: "HierNode" },
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
