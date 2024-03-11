use anyhow::Context;

use crate::ecs::component::Component;
use crate::ecs::resources::Resources;
use crate::ecs::storage::vec_storage::VecStorage;
use crate::engine::assets::cpu_model::CpuModel;
use crate::engine::assets::gpu_model::GpuModel;
use crate::engine::resources::asset_database::AssetDatabase;

#[derive(Debug)]
pub struct Renderable {
    pub model: GpuModel,
    pub group: String,
    pub name: String,
}

impl Renderable {
    pub async fn with_model<S: AsRef<str>>(res: &Resources, group: S, name: S) -> Result<Self, anyhow::Error> {
        let group = group.as_ref();
        let name = name.as_ref();
        let cpu_model = res
            .read::<AssetDatabase>()
            .load_asset::<CpuModel, _>(res, group, name)
            .await
            .with_context(|| format!("Loading CpuModel from group {} and name {}", group, name))?;
        let model = GpuModel::with_model(res, &cpu_model);
        Ok(Renderable {
            model,
            group: group.to_string(),
            name: name.to_string(),
        })
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}
