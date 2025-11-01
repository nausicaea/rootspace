#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LightUniform {
    pub model_view: [[f32; 4]; 4],
    pub color: [f32; 4],
}
