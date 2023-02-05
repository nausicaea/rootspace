use std::path::Path;

use crate::resources::graphics::{ids::BindGroupId, Graphics};

use super::{texture::Texture, Error};

#[derive(Debug)]
pub struct Material {
    pub texture: Texture,
    pub bind_group: BindGroupId,
}

impl Material {
    pub(crate) fn with_file(gfx: &mut Graphics, path: &Path) -> Result<Self, Error> {
        let texture = Texture::with_file(gfx, path)?;
        let bind_group = gfx
            .create_bind_group(gfx.material_layout())
            .add_texture_view(0, texture.view)
            .add_sampler(1, texture.sampler)
            .submit();

        Ok(Material { texture, bind_group })
    }
}
