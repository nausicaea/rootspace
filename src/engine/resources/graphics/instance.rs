use super::descriptors::VertexAttributeDescriptor;

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
    pub with_camera: f32,
}

impl VertexAttributeDescriptor for Instance {
    const STEP_MODE: wgpu::VertexStepMode = wgpu::VertexStepMode::Instance;
    const ATTRS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32];
}