use anyhow::anyhow;
use ecs::{Component, Resources, Storage, VecStorage};
use glamour::vec::Vec4;

use crate::base::gpu_model::GpuModel;
use crate::resources::Graphics;
use crate::utilities::load_instanced_gpu_model;

#[derive(Debug)]
pub struct Light {
    pub model: GpuModel,
    pub position: Vec4<f32>,
    pub ambient_color: Vec4<f32>,
    pub diffuse_color: Vec4<f32>,
    pub specular_color: Vec4<f32>,
    pub ambient_intensity: f32,
    pub point_intensity: f32,
    pub group: String,
    pub name: String,
}

impl Light {
    #[tracing::instrument(skip_all)]
    pub async fn new(res: &Resources, source: &LightSource) -> anyhow::Result<Self> {
        {
            let max_lights = res.read::<Graphics>().max_lights() as usize;
            let lights = res.read_components::<Light>();
            let num_lights = lights.len();
            if num_lights >= max_lights {
                return Err(anyhow!(
                    "The maximum number of light sources ({max_lights}) has been reached"
                ));
            }
        }

        let model = load_instanced_gpu_model(res, &source.group, &source.name).await?;

        Ok(Self {
            model,
            position: source.position,
            ambient_color: source.ambient_color,
            diffuse_color: source.diffuse_color,
            specular_color: source.specular_color,
            ambient_intensity: source.ambient_intensity,
            point_intensity: source.point_intensity,
            group: source.group.clone(),
            name: source.name.clone(),
        })
    }
}

impl Component for Light {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LightSource {
    pub group: String,
    pub name: String,
    pub position: Vec4<f32>,
    pub ambient_color: Vec4<f32>,
    pub diffuse_color: Vec4<f32>,
    pub specular_color: Vec4<f32>,
    pub ambient_intensity: f32,
    pub point_intensity: f32,
}
