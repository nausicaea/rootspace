use super::descriptors::VertexAttributeDescriptor;

#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Instance {
    pub model_view: [[f32; 4]; 4],
    pub normal: [[f32; 4]; 4],
    // TODO: this kind of handling of non-projected UI objects is janky
    pub with_camera: f32,
}

impl VertexAttributeDescriptor for Instance {
    const STEP_MODE: wgpu::VertexStepMode = wgpu::VertexStepMode::Instance;
    const ATTRS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![
            4 => Float32x4, 
            5 => Float32x4, 
            6 => Float32x4, 
            7 => Float32x4, 
            8 => Float32x4, 
            9 => Float32x4, 
            10 => Float32x4, 
            11 => Float32x4, 
            12 => Float32,
        ];
}
