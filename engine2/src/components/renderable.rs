use crate::assets::model::Model;

#[derive(Debug)]
pub struct Renderable(pub Model);

impl ecs::Component for Renderable {
    type Storage = ecs::VecStorage<Self>;
}
