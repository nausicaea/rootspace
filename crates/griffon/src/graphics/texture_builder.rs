use wgpu::util::TextureDataOrder;

use super::settings::Settings;
use super::{GpuObjectDatabase, ids::TextureId, runtime::Runtime};

pub struct TextureBuilder<'rt> {
    runtime: &'rt Runtime<'rt>,
    database: &'rt mut GpuObjectDatabase,
    settings: &'rt Settings,
    label: Option<&'rt str>,
    image: Option<&'rt image::DynamicImage>,
    depth_texture: bool,
}

impl<'rt> TextureBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut GpuObjectDatabase, settings: &'rt Settings) -> Self {
        Self {
            runtime,
            database,
            settings,
            label: None,
            image: None,
            depth_texture: false,
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

    pub fn with_depth_texture(mut self) -> Self {
        self.depth_texture = true;
        self
    }

    pub fn submit(self) -> TextureId {
        let texture = if self.depth_texture {
            tracing::trace!("Creating depth texture '{}'", self.label.unwrap_or("unnamed"));
            self.runtime.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("depth-texture"),
                size: wgpu::Extent3d {
                    width: self.runtime.config.width,
                    height: self.runtime.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.settings.depth_texture_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        } else {
            use wgpu::util::DeviceExt;

            let image = self.image.expect("cannot build a texture without a source image file");

            let rgba8_image = image.to_rgba8();

            let dims = rgba8_image.dimensions();

            tracing::trace!("Creating texture '{}'", self.label.unwrap_or("unnamed"));
            self.runtime.device.create_texture_with_data(
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
            )
        };

        self.database.insert_texture(texture)
    }
}
