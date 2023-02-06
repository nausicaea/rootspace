use std::path::Path;

use ecs::Resources;

use crate::resources::graphics::{ids::BindGroupId, Graphics};

use super::{texture::Texture, Asset, Error};

#[derive(Debug)]
pub struct Material {
    pub texture: Texture,
    pub bind_group: BindGroupId,
}

impl Asset for Material {
    fn with_path(res: &Resources, path: &Path) -> Result<Self, Error> {
        let texture = Texture::with_path(res, path)?;

        let mut gfx = res.borrow_mut::<Graphics>();
        let layout = gfx.material_layout();
        let bind_group = gfx
            .create_bind_group(layout)
            .add_texture_view(0, texture.view)
            .add_sampler(1, texture.sampler)
            .submit();

        Ok(Material { texture, bind_group })
    }
}
