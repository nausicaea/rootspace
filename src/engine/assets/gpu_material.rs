use std::path::Path;

use crate::ecs::resources::Resources;
use crate::engine::resources::graphics::ids::BindGroupId;
use crate::engine::resources::graphics::Graphics;
use anyhow::Context;

use super::{private::PrivLoadAsset, texture::Texture};

#[derive(Debug)]
pub struct GpuMaterial {
    pub texture: Texture,
    pub bind_group: BindGroupId,
}

impl PrivLoadAsset for GpuMaterial {
    type Output = Self;

    async fn with_path(res: &Resources, path: &Path) -> Result<Self::Output, anyhow::Error> {
        let texture = Texture::with_path(res, path).await.with_context(|| {
            format!(
                "trying to load a {} from '{}'",
                std::any::type_name::<Texture>(),
                path.display()
            )
        })?;

        let mut gfx = res.write::<Graphics>();
        let layout = gfx.material_layout();
        let bind_group = gfx
            .create_bind_group(layout)
            .add_texture_view(0, texture.view)
            .add_sampler(1, texture.sampler)
            .submit();

        Ok(GpuMaterial { texture, bind_group })
    }
}
