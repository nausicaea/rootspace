use crate::ecs::component::Component;
use crate::ecs::storage::vec_storage::VecStorage;
use crate::glamour::mat::Mat4;
use crate::glamour::persp::Persp;

pub mod projection;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera(Persp<f32>);

impl Camera {
    pub fn new(width: u32, height: u32, fov_y: f32, frustum_z: (f32, f32)) -> Self {
        Camera(Persp::new(
            height as f32 / width as f32,
            fov_y,
            frustum_z.0,
            frustum_z.1,
        ))
    }

    pub fn as_matrix(&self) -> &Mat4<f32> {
        self.0.as_matrix()
    }

    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        let aspect = height as f32 / width as f32;
        log::warn!("Aspect changed to: {aspect}");
        self.0.set_aspect(aspect);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera(Persp::new(600.0 / 800.0, std::f32::consts::PI / 4.0, 0.1, 1000.0))
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
