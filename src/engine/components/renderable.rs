use crate::ecs::component::Component;
use crate::ecs::storage::vec_storage::VecStorage;
use crate::engine::assets::model::Model;

#[derive(Debug)]
pub struct Renderable(pub Model);

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum RenderableSource {
    Model { group: String, name: String },
}

impl RenderableSource {
    pub fn with_model<S: AsRef<str>>(group: S, name: S) -> Self {
        RenderableSource::Model {
            group: group.as_ref().into(),
            name: name.as_ref().into(),
        }
    }
}
