use griffon::Graphics;
use griffon::base::ids::{SamplerId, TextureId, TextureViewId};

#[derive(Debug)]
pub struct Texture {
    #[allow(dead_code)]
    pub texture: TextureId,
    pub view: TextureViewId,
    pub sampler: SamplerId,
}

impl Texture {
    pub async fn from_file(graphics: &mut Graphics, file_name: &str) -> anyhow::Result<Self> {
        let data = crate::util::load_binary(file_name).await?;
        Self::from_bytes(graphics, &data, file_name)
    }

    pub fn from_bytes(graphics: &mut Graphics, bytes: &[u8], label: &str) -> anyhow::Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(graphics, &img, Some(label))
    }

    pub fn from_image(graphics: &mut Graphics, img: &image::DynamicImage, label: Option<&str>) -> anyhow::Result<Self> {
        let texture = graphics.create_texture().with_label(label).with_image(img).submit();

        Ok(Self {
            texture,
            view: graphics.create_texture_view(label, texture),
            sampler: graphics.create_sampler().with_label(label).submit(),
        })
    }
}
