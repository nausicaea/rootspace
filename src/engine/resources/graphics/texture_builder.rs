use wgpu::util::TextureDataOrder;

use super::{ids::TextureId, runtime::Runtime, Database};

pub struct TextureBuilder<'rt> {
    runtime: &'rt Runtime<'rt>,
    database: &'rt mut Database,
    label: Option<&'rt str>,
    image: Option<&'rt image::DynamicImage>,
}

impl<'rt> TextureBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database) -> Self {
        Self {
            runtime,
            database,
            label: None,
            image: None,
        }
    }

    pub fn with_label(mut self, label: Option<&'rt str>) -> Self {
        self.label = label;
        self
    }

    pub fn with_image(mut self, image: &'rt image::DynamicImage) -> Self {
        self.image = Some(image);
        self
    }

    pub fn submit(self) -> TextureId {
        use wgpu::util::DeviceExt;

        let image = self.image.expect("cannot build a texture without a source image file");

        let rgba8_image = image.to_rgba8();

        let dims = rgba8_image.dimensions();

        tracing::trace!("Creating texture '{}'", self.label.unwrap_or("unnamed"));
        let texture = self.runtime.device.create_texture_with_data(
            &self.runtime.queue,
            &wgpu::TextureDescriptor {
                label: self.label,
                size: wgpu::Extent3d {
                    width: dims.0,
                    height: dims.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
            },
            TextureDataOrder::default(),
            &rgba8_image,
        );

        self.database.insert_texture(texture)
    }
}
