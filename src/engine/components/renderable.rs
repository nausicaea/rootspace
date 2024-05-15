use anyhow::Context;

use crate::engine::resources::graphics::gpu_model::GpuModel;
use crate::engine::resources::graphics::Graphics;
use crate::{
    ecs::{component::Component, resources::Resources, storage::vec_storage::VecStorage},
    engine::{assets::cpu_model::CpuModel, resources::asset_database::AssetDatabase},
};

#[derive(Debug)]
pub struct Renderable {
    pub model: GpuModel,
    pub group: String,
    pub name: String,
}

impl Renderable {
    #[tracing::instrument(skip_all)]
    pub async fn with_model<S: AsRef<str> + std::fmt::Debug>(
        res: &Resources,
        group: S,
        name: S,
    ) -> Result<Self, anyhow::Error> {
        let group = group.as_ref();
        let name = name.as_ref();
        let instancing_candidate = res
            .iter_r::<Renderable>()
            .find(|(_, ren)| ren.group == group && ren.name == name);
        let model = if let Some((_, ren)) = instancing_candidate {
            res.write::<Graphics>().create_instanced_model(&ren.model)
        } else {
            let cpu_model = res
                .read::<AssetDatabase>()
                .load_asset::<CpuModel, _>(res, group, name)
                .await
                .with_context(|| format!("Loading CpuModel from group {} and name {}", group, name))?;
            res.write::<Graphics>().create_model(&cpu_model)
        };

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
