use crate::resources::graphics::{ids::BindGroupId, Graphics};

use super::{texture::Texture, Asset, Error};

#[derive(Debug)]
pub struct Material {
    pub texture: Texture,
    pub bind_group: BindGroupId,
}

impl Asset for Material {
    type Error = Error;

    fn with_file<S: AsRef<str>>(
        adb: &crate::resources::asset_database::AssetDatabase,
        gfx: &mut Graphics,
        group: S,
        name: S,
    ) -> Result<Self, Self::Error> {
        let texture = Texture::with_file(adb, gfx, group, name)?;
        let bind_group = gfx
            .create_bind_group(gfx.material_layout())
            .add_texture_view(0, texture.view)
            .add_sampler(1, texture.sampler)
            .submit();

        Ok(Material { texture, bind_group })
    }
}
