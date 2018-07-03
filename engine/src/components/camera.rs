use super::AsMatrix;
use nalgebra::Matrix4;

#[derive(Debug)]
pub struct Camera(Matrix4<f32>);

impl Default for Camera {
    fn default() -> Self {
        Camera(Matrix4::identity())
    }
}

impl AsMatrix for Camera {
    fn as_matrix(&self) -> &Matrix4<f32> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_matrix() {
        let c = Camera::default();
        let _: &Matrix4<f32> = c.as_matrix();
    }
}
