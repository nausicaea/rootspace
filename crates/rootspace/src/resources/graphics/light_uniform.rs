#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LightUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
}
