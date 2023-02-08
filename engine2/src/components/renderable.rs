use crate::assets::model::Model;

#[derive(Debug)]
pub struct Renderable(pub Model);

impl ecs::Component for Renderable {
    type Storage = ecs::VecStorage<Self>;
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
