pub mod projection;

use ecs::{Component, VecStorage};
use glamour::{Mat4, Persp};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera(Persp<f32>);

impl Camera {
    pub fn new(dimensions: (u32, u32), fov_y: f32, frustum_z: (f32, f32)) -> Self {
        Camera(Persp::new(
            dimensions.0 as f32 / dimensions.1 as f32,
            fov_y,
            frustum_z.0,
            frustum_z.1,
        ))
    }

    pub fn as_matrix(&self) -> &Mat4<f32> {
        self.0.as_matrix()
    }

    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) {
        self.0.set_aspect(dimensions.0 as f32 / dimensions.1 as f32);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera(Persp::new(800.0 / 600.0, std::f32::consts::PI / 4.0, 0.1, 1000.0))
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
