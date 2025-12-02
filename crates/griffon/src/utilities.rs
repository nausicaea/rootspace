use crate::Graphics;
use crate::assets::cpu_model::CpuModel;
use crate::base::gpu_model::GpuModel;
use crate::components::renderable::Renderable;
use anyhow::Context;
use assam::AssetDatabase;
use ecs::resources::Resources;

/// Load a new [`GpuModel`] from an asset known to [`AssetDatabase`]. Automatically instance the model if it is already present in the model database of [`Graphics`] by searching for matching [`Renderable`].
pub async fn load_instanced_gpu_model(res: &Resources, group: &str, name: &str) -> anyhow::Result<GpuModel> {
    let instancing_candidate = res
        .iter_r::<Renderable>()
        .find(|(_, ren)| &ren.group == group && &ren.name == name);

    if let Some((_, ren)) = instancing_candidate {
        Ok(res.write::<Graphics>().create_instanced_gpu_model(&ren.model))
    } else {
        let cpu_model = res
            .read::<AssetDatabase>()
            .load_asset::<CpuModel, _>(res, group, name)
            .await
            .with_context(|| format!("Loading CpuModel from group {} and name {}", group, name))?;
        Ok(res.write::<Graphics>().create_gpu_model(&cpu_model))
    }
}
