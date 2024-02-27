#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub features: wgpu::Features,
    pub limits: wgpu::Limits,
    pub preferred_texture_format: wgpu::TextureFormat,
    pub present_mode: wgpu::PresentMode,
    pub alpha_mode: wgpu::CompositeAlphaMode,
    pub clear_color: wgpu::Color,
    pub max_objects: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::LowPower,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            preferred_texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            max_objects: 1 << 10,
        }
    }
}
