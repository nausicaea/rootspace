#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Renderable;

impl ecs::Component for Renderable {
    type Storage = ecs::VecStorage<Self>;
}
