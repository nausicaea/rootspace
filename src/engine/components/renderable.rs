use crate::ecs::component::Component;
use crate::ecs::storage::vec_storage::VecStorage;
use crate::engine::assets::model::Model;

#[derive(Debug)]
pub struct Renderable(pub Model);

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}

