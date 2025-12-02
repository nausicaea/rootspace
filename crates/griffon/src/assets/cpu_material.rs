use super::cpu_texture::CpuTexture;
use anyhow::Context;
use assam::LoadAsset;
use tracing::warn;

#[derive(Debug)]
pub struct CpuMaterial {
    pub label: Option<String>,
    pub texture: CpuTexture,
    pub ambient_reflectivity: f32,
    pub diffuse_reflectivity: f32,
    pub specular_reflectivity: f32,
    pub smoothness: f32,
}

impl LoadAsset for CpuMaterial {
    type Output = Self;

    #[tracing::instrument(skip(res))]
    async fn with_path(res: &ecs::resources::Resources, path: &std::path::Path) -> anyhow::Result<Self::Output> {
        let label = path.file_stem().and_then(|n| n.to_str()).map(|n| n.to_owned());
        let texture = CpuTexture::with_path(res, path)
            .await
            .with_context(|| format!("Loading CpuTexture at path {}", path.display()))?;

        warn!("Use of hard-coded material properties in CpuMaterial");
        Ok(CpuMaterial {
            label,
            texture,
            ambient_reflectivity: 1.0,
            diffuse_reflectivity: 1.0,
            specular_reflectivity: 1.0,
            smoothness: 32.0,
        })
    }
}
