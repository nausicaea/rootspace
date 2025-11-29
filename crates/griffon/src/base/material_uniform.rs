#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct MaterialUniform {
    pub ambient_reflectivity: f32,
    pub diffuse_reflectivity: f32,
    pub specular_reflectivity: f32,
    pub smoothness: f32,
}
