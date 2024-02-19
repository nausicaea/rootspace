use image::ImageFormat;
use crate::ecs::resources::Resources;
use crate::engine::resources::graphics::Graphics;
use crate::engine::resources::graphics::ids::{SamplerId, TextureId, TextureViewId};

use super::{private::PrivLoadAsset, Error};

#[derive(Debug)]
pub struct Texture {
    pub texture: TextureId,
    pub view: TextureViewId,
    pub sampler: SamplerId,
}

impl PrivLoadAsset for Texture {
    type Output = Self;

    fn with_path(res: &Resources, path: &std::path::Path) -> Result<Self::Output, anyhow::Error> {
        let image_format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
                "png" => Some(ImageFormat::Png),
                _ => None,
            })
            .ok_or(Error::UnsupportedFileFormat)?;

        let f = std::fs::File::open(path)?;
        let img = image::load(std::io::BufReader::new(f), image_format)?;

        let mut gfx = res.borrow_mut::<Graphics>();
        let texture = gfx.create_texture().with_image(img).submit();
        let view = gfx.create_texture_view(None, texture);
        let sampler = gfx.create_sampler().submit();

        Ok(Texture { texture, view, sampler })
    }
}
