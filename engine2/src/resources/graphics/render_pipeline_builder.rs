use super::{
    descriptors::VertexAttributeDescriptor,
    ids::{BindGroupLayoutId, PipelineId, ShaderModuleId},
    indexes::Indexes,
    runtime::Runtime,
    tables::Tables,
};

#[derive(Debug)]
pub struct RenderPipelineBuilder<'rt, 'l, 'vbl, 'lbl> {
    runtime: &'rt Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
    vertex_shader_module: Option<(ShaderModuleId, &'l str)>,
    fragment_shader_module: Option<(ShaderModuleId, &'l str)>,
    bind_group_layouts: Vec<BindGroupLayoutId>,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vbl>>,
    label: Option<&'lbl str>,
}

impl<'rt, 'l, 'vbl, 'lbl> RenderPipelineBuilder<'rt, 'l, 'vbl, 'lbl> {
    pub(super) fn new(runtime: &'rt Runtime, indexes: &'rt mut Indexes, tables: &'rt mut Tables) -> Self {
        RenderPipelineBuilder {
            runtime,
            indexes,
            tables,
            vertex_shader_module: None,
            fragment_shader_module: None,
            bind_group_layouts: Vec::new(),
            vertex_buffer_layouts: Vec::new(),
            label: None,
        }
    }

    pub fn with_label(mut self, label: &'lbl str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_vertex_shader_module(mut self, module: ShaderModuleId, entry_point: &'l str) -> Self {
        self.vertex_shader_module = Some((module, entry_point));
        self
    }

    pub fn with_fragment_shader_module(mut self, module: ShaderModuleId, entry_point: &'l str) -> Self {
        self.fragment_shader_module = Some((module, entry_point));
        self
    }

    pub fn add_bind_group_layout(mut self, bgl: BindGroupLayoutId) -> Self {
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

    pub fn submit(self) -> PipelineId {
        // Helper variables
        let cts = [Some(wgpu::ColorTargetState {
            format: self.runtime.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        dbg!(&self.tables.bind_group_layouts);
        let bgl = self
            .bind_group_layouts
            .into_iter()
            .map(|b| {
                self.tables
                    .bind_group_layouts
                    .get(&b)
                    .expect(&format!("Unknown {:?}", b))
            })
            .collect::<Vec<_>>();
        let pipeline_layout = self
            .runtime
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bgl,
                push_constant_ranges: &[],
            });

        // Pipeline definition
        let pipeline = self
            .runtime
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: self.label,
                layout: Some(&pipeline_layout),
                vertex: self
                    .vertex_shader_module
                    .map(|(vsm, vep)| wgpu::VertexState {
                        module: self
                            .tables
                            .shader_modules
                            .get(&vsm)
                            .expect(&format!("Unknown {:?}", vsm)),
                        entry_point: vep,
                        buffers: self.vertex_buffer_layouts.as_slice(),
                    })
                    .expect("cannot build a render pipeline without vertex shader module"),
                fragment: self.fragment_shader_module.map(|(fsm, fep)| wgpu::FragmentState {
                    module: self
                        .tables
                        .shader_modules
                        .get(&fsm)
                        .expect(&format!("Unknown {:?}", fsm)),
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
        log::trace!("Registering {:?} as {:?}", &pipeline, id);
        self.tables.render_pipelines.insert(id, pipeline);
        id
    }
}
