use anyhow::{anyhow, Context};
use ecs::{Resource, SerializationName};
use file_manipulation::{FilePathBuf, FileError};
use try_default::TryDefault;

use tokio::runtime::Runtime;
use serde::{
    Serialize,
    Deserialize,
    ser::Serializer,
    de::{Error as DeError, Deserializer},
};

use wgpu;

#[derive(Debug)]
pub struct Graphics {
    builder: GraphicsBuilder,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
}

impl Graphics {
    pub fn builder() -> GraphicsBuilder {
        GraphicsBuilder::default()
    }
}

impl TryDefault for Graphics {
    fn try_default() -> Result<Self, anyhow::Error> {
        Self::builder().build()
    }
}

impl Resource for Graphics {}

impl SerializationName for Graphics {}

impl Serialize for Graphics {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
        self.builder.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Graphics {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        GraphicsBuilder::deserialize(d)?
            .build()
            .map_err(|e| DeError::custom(format!("{}", e)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsBuilder {
    backends: wgpu::Backends,
    power_preference: wgpu::PowerPreference,
    features: wgpu::Features,
    limits: wgpu::Limits,
    texture_format: wgpu::TextureFormat,
    shader: Option<Shader>,
}

impl GraphicsBuilder {
    pub fn with_backends(mut self, backends: wgpu::Backends) -> Self {
        self.backends = backends;
        self
    }

    pub fn with_power_preference(mut self, power_preference: wgpu::PowerPreference) -> Self {
        self.power_preference = power_preference;
        self
    }

    pub fn with_features(mut self, features: wgpu::Features) -> Self {
        self.features = features;
        self
    }

    pub fn with_limits(mut self, limits: wgpu::Limits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_texture_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.texture_format = format;
        self
    }

    pub fn with_shader(mut self, shader: Shader) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn build(self) -> Result<Graphics, anyhow::Error> {
        let shader_data = self.shader
            .as_ref()
            .ok_or(anyhow!("You must provide a shader with GraphicsBuilder::with_shader"))?;

        // Create the runtime
        let rt  = Runtime::new()
            .context("Unable to create the tokio runtime")?;

        let instance = wgpu::Instance::new(self.backends);

        let adapter = rt.block_on(instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
            .ok_or(anyhow!("Unable to retrieve the requested adapter"))?;

        let (device, queue) = rt.block_on(adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: self.features,
                    limits: self.limits.clone(),
                    label: Some("Device"),
                },
                None,
            ))
            .context("Unable to retrieve device and queue")?;

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[todo!()],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: shader_data.source.to_shader_source().context("Cannot load shader data")?,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: &shader_data.vertex_entry_point,
                buffers: &[todo!()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: &shader_data.fragment_entry_point,
                targets: &shader_data.targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(Graphics {
            builder: self, device, queue, render_pipeline,
        })
    }
}

impl Default for GraphicsBuilder {
    fn default() -> Self {
        GraphicsBuilder {
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::LowPower,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            shader: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shader {
    pub source: ShaderSource,
    pub vertex_entry_point: String,
    pub fragment_entry_point: String,
    pub targets: Vec<wgpu::ColorTargetState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderSource {
    #[cfg(feature = "spirv")]
    SpirV(FilePathBuf),
    Wgsl(FilePathBuf),
}

impl ShaderSource {
    pub fn to_shader_source(&self) -> Result<wgpu::ShaderSource, FileError> {
        self.clone().try_into()
    }
}

impl<'a> TryInto<wgpu::ShaderSource<'a>> for ShaderSource {
    type Error = FileError;

    fn try_into(self) -> Result<wgpu::ShaderSource<'a>, Self::Error> {
        match self {
            ShaderSource::Wgsl(fp) => {
                let data = fp.read_to_string()?;
                Ok(wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(data)))
            }
            #[cfg(feature = "spirv")]
            ShaderSource::SpirV(_) => {
                todo!()
            }
        }
    }
}
