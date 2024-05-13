use super::descriptors::VertexAttributeDescriptor;

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
}

impl VertexAttributeDescriptor for Instance {
    const ATTRS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4];
    const STEP_MODE: wgpu::VertexStepMode = wgpu::VertexStepMode::Instance;
}
