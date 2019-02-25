use std::ops::Mul;
use ecs::{Entity, Resource, VecStorage};
use failure::Error;
use hierarchy::{Hierarchy, RawNodes};
use std::fmt;

#[derive(Default)]
pub struct SceneGraph<T>
where
    T: Clone + Default,
{
    hierarchy: Hierarchy<Entity, T>,
}

impl<T> SceneGraph<T>
where
    T: Clone + Default,
    for<'r> &'r T: Mul<&'r T, Output = T>,
{
    pub fn update(&mut self, data: &VecStorage<T>) {
        // let db = &self.database;
        // self.world_graph.update(&|entity, _, parent_model| {
        //     let current_model: &Model = db.get(entity).ok()?;
        //     Some(parent_model * current_model)
        // })?;
        // self.ui_graph.update(&|entity, _, parent_model| {
        //     let current_model: &UiModel = db.get(entity).ok()?;
        //     Some(parent_model * current_model)
        // })?;
        self.hierarchy.update(&|entity, _, parent_datum| {
            data.get(entity)
                .map(|current_datum| parent_datum * current_datum)
        })
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
        write!(f, "SceneGraph {{ ... }}")
    }
}

impl<T> Resource for SceneGraph<T> where T: 'static + Clone + Default {}
