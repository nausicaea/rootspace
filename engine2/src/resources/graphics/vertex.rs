use super::descriptors::VertexAttributeDescriptor;

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) normals: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
}

impl VertexAttributeDescriptor for Vertex {
    const STEP_MODE: wgpu::VertexStepMode = wgpu::VertexStepMode::Vertex;
    const ATTRS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
}
