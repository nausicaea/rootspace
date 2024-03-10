use crate::ecs::component::Component;
use crate::ecs::storage::vec_storage::VecStorage;
use crate::engine::assets::gpu_model::GpuModel;

#[derive(Debug)]
pub struct Renderable {
    pub model: GpuModel,
    pub group: String,
    pub name: String,
}

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}
