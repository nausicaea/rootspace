use super::{ids::{PipelineId, BindGroupLayoutId}, runtime::Runtime, tables::Tables, indexes::Indexes, vertex_attribute_descriptor::VertexAttributeDescriptor};

#[derive(Debug)]
pub struct RenderPipelineBuilder<'rt, 'l, 'bgl, 'vbl> {
    runtime: &'rt mut Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
    shader_module: Option<wgpu::ShaderModule>,
    vertex_entry_point: Option<&'l str>,
    fragment_entry_point: Option<&'l str>,
    bind_group_layouts: Vec<&'bgl BindGroupLayoutId>,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vbl>>,
}

impl<'rt, 'l, 'bgl, 'vbl> RenderPipelineBuilder<'rt, 'l, 'bgl, 'vbl> {
    pub(super) fn new(runtime: &'rt mut Runtime, indexes: &'rt mut Indexes, tables: &'rt mut Tables) -> Self {
        RenderPipelineBuilder {
            runtime,
            indexes,
            tables,
            shader_module: None,
            vertex_entry_point: None,
            fragment_entry_point: None,
            bind_group_layouts: Vec::new(),
            vertex_buffer_layouts: Vec::new(),
        }
    }

    pub fn with_shader_module<'s, S: Into<std::borrow::Cow<'s, str>>>(
        mut self,
        label: Option<&str>,
        source: S,
        vertex_entry_point: &'l str,
        fragment_entry_point: Option<&'l str>,
    ) -> Self {
        let sm = self.runtime.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.shader_module = Some(sm);
        self.vertex_entry_point = Some(vertex_entry_point);
        self.fragment_entry_point = fragment_entry_point;
        self
    }

    pub fn add_bind_group_layout(mut self, bgl: &'bgl BindGroupLayoutId) -> Self {
        self.bind_group_layouts.push(bgl);
        self
    }

    pub fn add_vertex_buffer_layout<V: VertexAttributeDescriptor>(mut self) -> Self {
        let vbl = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<V>() as wgpu::BufferAddress,
            step_mode: V::STEP_MODE,
            attributes: V::ATTRS,
        };
        self.vertex_buffer_layouts.push(vbl);
        self
    }

    pub fn submit(self, label: Option<&str>) -> Result<PipelineId, Error> {
        // Required parameters
        let shader_module = self.shader_module.ok_or(Error::MissingShaderModule)?;
        let vertex_entry_point = self.vertex_entry_point.ok_or(Error::MissingShaderModule)?;

        // Helper variables
        let cts = [Some(wgpu::ColorTargetState {
            format: self.runtime.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let bgl = self.bind_group_layouts.into_iter()
            .map(|b| &self.tables.bind_group_layouts[b])
            .collect::<Vec<_>>();
        let pipeline_layout = self
            .runtime
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bgl,
                push_constant_ranges: &[],
            });

        let pipeline = self
            .runtime
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: vertex_entry_point,
                    buffers: self.vertex_buffer_layouts.as_slice(),
                },
                fragment: self.fragment_entry_point.map(|fep| wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: fep,
                    targets: &cts,
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

        let id = self.indexes.render_pipelines.take();
        self.tables.render_pipelines.insert(id, pipeline);

        Ok(id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot build a render pipeline without shader module")]
    MissingShaderModule,
}
