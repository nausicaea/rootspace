use ecs::{Component, Entity, Storage};
use hierarchy::{Hierarchy, RawNodes};
use serde::{Deserialize, Serialize};
use std::{fmt, ops::Mul};
use typename::TypeName;

#[derive(Default, TypeName, Serialize, Deserialize)]
pub struct SceneGraph<T>
where
    T: Clone + Default,
{
    hierarchy: Hierarchy<Entity, T>,
}

impl<T> SceneGraph<T>
where
    T: Clone + Default + Component,
    for<'r> &'r T: Mul<&'r T, Output = T>,
{
    pub fn update(&mut self, data: &<T as Component>::Storage) {
        self.hierarchy
            .update(&|entity, _, parent_datum| data.get(entity).map(|current_datum| parent_datum * current_datum))
    }

    pub fn insert(&mut self, entity: Entity) {
        self.hierarchy.insert(entity, Default::default())
    }

    pub fn get(&self, entity: &Entity) -> Option<&T> {
        self.hierarchy
            .iter()
            .filter(|&(k, _)| k == entity)
            .map(|(_, v)| v)
            .last()
    }

    pub fn iter(&self) -> RawNodes<Entity, T> {
        self.hierarchy.iter()
    }
}

impl<T> fmt::Debug for SceneGraph<T>
where
    T: Clone + Default,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SceneGraph(#nodes: {})", self.hierarchy.len())
    }
}
