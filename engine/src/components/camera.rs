use nalgebra::{Isometry3, Matrix4, Orthographic3, Perspective3, Point3, Vector3};
use std::f32;

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    world_matrix: Matrix4<f32>,
    ui_matrix: Matrix4<f32>,
    orthographic: Orthographic3<f32>,
    perspective: Perspective3<f32>,
    view: Isometry3<f32>,
    dimensions: (u32, u32),
    dpi_factor: f64,
}

impl Camera {
    pub fn new(
        dimensions: (u32, u32),
        fov_y: f32,
        frustum_z: (f32, f32),
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
    ) -> Self {
        let orthographic = Orthographic3::new(
            dimensions.0 as f32 / -2.0,
            dimensions.0 as f32 / 2.0,
            dimensions.1 as f32 / -2.0,
            dimensions.1 as f32 / 2.0,
            frustum_z.0,
            frustum_z.1,
        );
        let perspective = Perspective3::new(
            dimensions.0 as f32 / dimensions.1 as f32,
            fov_y,
            frustum_z.0,
            frustum_z.1,
        );
        let view = Isometry3::look_at_rh(&eye, &target, &up);

        Camera {
            world_matrix: perspective.as_matrix() * view.to_homogeneous(),
            ui_matrix: orthographic.as_matrix().clone(),
            orthographic,
            perspective,
            view,
            dimensions,
            dpi_factor: 1.0,
        }
    }

    pub fn world_matrix(&self) -> &Matrix4<f32> {
        &self.world_matrix
    }

    pub fn ui_matrix(&self) -> &Matrix4<f32> {
        &self.ui_matrix
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.view.translation.vector
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    pub fn physical_dimensions(&self) -> (u32, u32) {
        let (w, h) = (self.dimensions.0 as f64, self.dimensions.1 as f64);
        (
            (w * self.dpi_factor).round() as u32,
            (h * self.dpi_factor).round() as u32,
        )
    }

    pub fn set_dimensions(&mut self, value: (u32, u32)) {
        if value != self.dimensions {
            self.perspective.set_aspect(value.0 as f32 / value.1 as f32);
            self.dimensions = value;
            self.recalculate_matrices();
        }
    }

    pub fn dpi_factor(&self) -> f64 {
        self.dpi_factor
    }

    pub fn set_dpi_factor(&mut self, value: f64) {
        self.dpi_factor = value;
    }

    fn recalculate_matrices(&mut self) {
        self.world_matrix = self.perspective.as_matrix() * self.view.to_homogeneous();
        self.ui_matrix = self.orthographic.as_matrix().clone();
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new(
            (800, 600),
            f32::consts::PI / 4.0,
            (0.1, 1000.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _ = Camera::new(
            (800, 600),
            f32::consts::PI / 4.0,
            (0.1, 1000.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y(),
        );
    }

    #[test]
    fn default() {
        assert_eq!(
            Camera::default(),
            Camera::new(
                (800, 600),
                f32::consts::PI / 4.0,
                (0.1, 1000.0),
                Point3::new(0.0, 0.0, 1.0),
                Point3::new(0.0, 0.0, -1.0),
                Vector3::y()
            )
        );
    }

    #[test]
    fn accessors() {
        let mut c = Camera::default();

        c.set_dimensions((1024, 768));
        assert_eq!(c.dpi_factor(), 1.0);
        assert_eq!(c.dimensions(), (1024, 768));
        assert_eq!(c.physical_dimensions(), (1024, 768));

        c.set_dpi_factor(2.0);
        assert_eq!(c.dpi_factor(), 2.0);
        assert_eq!(c.dimensions(), (1024, 768));
        assert_eq!(c.physical_dimensions(), (2048, 1536));
    }

    #[test]
    fn world_matrix_accessor() {
        let c = Camera::default();
        let m: &Matrix4<f32> = c.world_matrix();
        assert_eq!(m, &(c.perspective.as_matrix() * c.view.to_homogeneous()));
    }

    #[test]
    fn ui_matrix_accessor() {
        let c = Camera::default();
        let m: &Matrix4<f32> = c.ui_matrix();
        assert_eq!(m, c.orthographic.as_matrix());
    }
}
