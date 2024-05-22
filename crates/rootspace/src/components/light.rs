use anyhow::Context;
use ecs::{component::Component, resources::Resources, storage::vec_storage::VecStorage};
use glamour::vec::Vec4;

use crate::{assets::cpu_model::CpuModel, resources::graphics::{gpu_model::GpuModel, Graphics}, AssetDatabase};

#[derive(Debug)]
pub struct Light {
    pub model: GpuModel,
    pub position: Vec4<f32>,
    pub color: Vec4<f32>,
    pub group: String,
    pub name: String,
}

impl Light {
    #[tracing::instrument(skip_all)]
    pub async fn with_model<S: AsRef<str> + std::fmt::Debug>(
        res: &Resources,
        group: S,
        name: S,
        position: Vec4<f32>,
        color: Vec4<f32>,
    ) -> Result<Self, anyhow::Error> {
        let group = group.as_ref();
        let name = name.as_ref();
        let cpu_model = res
            .read::<AssetDatabase>()
            .load_asset::<CpuModel, _>(res, group, name)
            .await
            .with_context(|| format!("Loading CpuModel from group {} and name {}", group, name))?;
        let model = res.write::<Graphics>().create_model(&cpu_model);

        Ok(Self {
            model,
            position,
            color,
            group: group.to_string(),
            name: name.to_string(),
        })
    }
}

impl Component for Light {
    type Storage = VecStorage<Self>;
}
