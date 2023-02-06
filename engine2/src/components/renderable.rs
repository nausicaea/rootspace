use crate::assets::model::Model;

#[derive(Debug)]
pub struct Renderable(pub Model);

impl ecs::Component for Renderable {
    type Storage = ecs::VecStorage<Self>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum RenderableSource {
    Model { group: String, name: String },
}
