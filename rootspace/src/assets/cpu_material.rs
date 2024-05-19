use anyhow::Context;

use super::{cpu_texture::CpuTexture, private::PrivLoadAsset};

#[derive(Debug)]
pub struct CpuMaterial {
    pub label: Option<String>,
    pub texture: CpuTexture,
}

impl PrivLoadAsset for CpuMaterial {
    type Output = Self;

    async fn with_path(
        res: &ecs::resources::Resources,
        path: &std::path::Path,
    ) -> Result<Self::Output, anyhow::Error> {
        let label = path.file_stem().and_then(|n| n.to_str()).map(|n| n.to_owned());
        let texture = CpuTexture::with_path(res, path)
            .await
            .with_context(|| format!("Loading CpuTexture at path {}", path.display()))?;

        Ok(CpuMaterial { label, texture })
    }
}
