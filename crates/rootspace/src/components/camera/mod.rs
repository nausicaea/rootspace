use ecs::{Component, VecStorage};
use glamour::{mat::Mat4, ortho::Ortho, persp::Persp};

pub mod projection;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera {
    persp: Persp<f32>,
    ortho: Ortho<f32>,
}

impl Camera {
    pub fn new(width: u32, height: u32, fov_y: f32, frustum_z: (f32, f32)) -> Self {
        Camera {
            persp: Persp::new(height as f32 / width as f32, fov_y, frustum_z.0, frustum_z.1),
            ortho: Ortho::new(width as f32, height as f32, frustum_z.0, frustum_z.1),
        }
    }

    pub fn as_persp_matrix(&self) -> &Mat4<f32> {
        self.persp.as_matrix()
    }

    pub fn as_ortho_matrix(&self) -> &Mat4<f32> {
        self.ortho.as_matrix()
    }

    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.persp.set_aspect(height as f32 / width as f32);
        self.ortho.set_dimensions(width as f32, height as f32);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            persp: Persp::new(1.0, std::f32::consts::PI / 4.0, 0.1, 1000.0),
            ortho: Ortho::new(800.0, 600.0, 0.0, 10.0),
        }
    }
}

impl Component for Camera {
    type Storage = VecStorage<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implements_default() {
        let _: Camera = Default::default();
    }
}
