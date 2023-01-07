pub trait VertexAttributeDescriptor {
    const STEP_MODE: wgpu::VertexStepMode;
    const ATTRS: &'static [wgpu::VertexAttribute];
}

