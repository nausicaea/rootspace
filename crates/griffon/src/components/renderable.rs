use crate::base::gpu_model::GpuModel;
use crate::utilities::load_instanced_gpu_model;
use ecs::{Component, Resources, VecStorage};

#[derive(Debug)]
pub struct Renderable {
    pub model: GpuModel,
    pub group: String,
    pub name: String,
}

impl Renderable {
    #[tracing::instrument(skip(res))]
    pub fn new(res: &Resources, source: &RenderableSource) -> anyhow::Result<Self> {
        let model = load_instanced_gpu_model(res, &source.group, &source.name)?;

        Ok(Renderable {
            model,
            group: source.group.clone(),
            name: source.name.clone(),
        })
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RenderableSource {
    pub group: String,
    pub name: String,
}
