use super::{ids::TextureId, indexes::Indexes, runtime::Runtime, tables::Tables};

pub struct TextureBuilder<'rt> {
    runtime: &'rt Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
    image: Option<image::DynamicImage>,
}

impl<'rt> TextureBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, indexes: &'rt mut Indexes, tables: &'rt mut Tables) -> Self {
        Self {
            runtime,
            indexes,
            tables,
            image: None,
        }
    }

    pub fn with_image(mut self, image: image::DynamicImage) -> Self {
        self.image = Some(image);
        self
    }

    pub fn submit(self, label: Option<&str>) -> TextureId {
        use wgpu::util::DeviceExt;

        let image = self.image.expect("cannot build a texture without a source image file");

        let rgba8_image = image.to_rgba8();

        let dims = rgba8_image.dimensions();

        let texture = self.runtime.device.create_texture_with_data(
            &self.runtime.queue,
            &wgpu::TextureDescriptor {
                label,
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
            },
            &rgba8_image,
        );

        let id = self.indexes.textures.take();
        self.tables.textures.insert(id, texture);
        id
    }
}
