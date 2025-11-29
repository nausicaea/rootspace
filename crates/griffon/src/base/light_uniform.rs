#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LightUniform {
    pub model_view: [[f32; 4]; 4],
    pub ambient_color: [f32; 4],
    pub specular_color: [f32; 4],
    pub ambient_intensity: f32,
    pub point_intensity: f32,
    pub _padding: [f32; 2],
}
