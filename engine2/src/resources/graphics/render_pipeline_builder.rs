use super::{
    descriptors::VertexAttributeDescriptor,
    ids::{BindGroupLayoutId, PipelineId, ShaderModuleId},
    runtime::Runtime,
    Database,
};

#[derive(Debug)]
pub struct RenderPipelineBuilder<'rt, 'ep, 'vbl> {
    runtime: &'rt Runtime,
    database: &'rt mut Database,
    vertex_shader_module: Option<(ShaderModuleId, &'ep str)>,
    fragment_shader_module: Option<(ShaderModuleId, &'ep str)>,
    bind_group_layouts: Vec<BindGroupLayoutId>,
    vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'vbl>>,
    label: Option<&'static str>,
}

impl<'rt, 'ep, 'vbl> RenderPipelineBuilder<'rt, 'ep, 'vbl> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database) -> Self {
        RenderPipelineBuilder {
            runtime,
            database,
            vertex_shader_module: None,
            fragment_shader_module: None,
            bind_group_layouts: Vec::new(),
            vertex_buffer_layouts: Vec::new(),
            label: None,
        }
    }

    pub fn with_label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_vertex_shader_module(mut self, module: ShaderModuleId, entry_point: &'ep str) -> Self {
        self.vertex_shader_module = Some((module, entry_point));
        self
    }

    pub fn with_fragment_shader_module(mut self, module: ShaderModuleId, entry_point: &'ep str) -> Self {
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
        let bgl = self
            .bind_group_layouts
            .into_iter()
            .map(|b| {
                self.database
                    .bind_group_layouts
                    .get(&b)
                    .expect(&format!("Unknown {:?}", b))
            })
            .collect::<Vec<_>>();

        // Pipeline layout definition
        let label_pipeline_layout = self.label.map(|lbl| format!("{}:pipeline-layout", lbl));
        log::trace!(
            "Creating pipeline layout '{}'",
            label_pipeline_layout.as_deref().unwrap_or("unnamed")
        );
        let pipeline_layout = self
            .runtime
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: label_pipeline_layout.as_deref(),
                bind_group_layouts: &bgl,
                push_constant_ranges: &[],
            });

        // Pipeline definition
        log::trace!("Creating render pipeline '{}'", self.label.unwrap_or("unnamed"));
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
                            .database
                            .shader_modules
                            .get(&vsm)
                            .expect(&format!("Unknown {:?}", vsm)),
                        entry_point: vep,
                        buffers: self.vertex_buffer_layouts.as_slice(),
                    })
                    .expect("cannot build a render pipeline without vertex shader module"),
                fragment: self.fragment_shader_module.map(|(fsm, fep)| wgpu::FragmentState {
                    module: self
                        .database
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

        self.database.insert_render_pipeline(self.label, pipeline)
    }
}
