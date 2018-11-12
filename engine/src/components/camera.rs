use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, Vector3};
use std::f32;

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    matrix: Matrix4<f32>,
    projection: Perspective3<f32>,
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
        let projection = Perspective3::new(
            dimensions.0 as f32 / dimensions.1 as f32,
            fov_y,
            frustum_z.0,
            frustum_z.1,
        );
        let view = Isometry3::look_at_rh(&eye, &target, &up);

        Camera {
            matrix: projection.as_matrix() * view.to_homogeneous(),
            projection,
            view,
            dimensions,
            dpi_factor: 1.0,
        }
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    pub fn set_dimensions(&mut self, value: (u32, u32)) {
        if value != self.dimensions {
            self.projection.set_aspect(value.0 as f32 / value.1 as f32);
            self.dimensions = value;
            self.recalculate_matrix();
        }
    }

    pub fn physical_dimensions(&self) -> (u32, u32) {
        let (w, h) = (self.dimensions.0 as f64, self.dimensions.1 as f64);
        ((w * self.dpi_factor).round() as u32, (h * self.dpi_factor).round() as u32)
    }

    pub fn set_dpi_factor(&mut self, value: f64) {
        self.dpi_factor = value;
    }

    fn recalculate_matrix(&mut self) {
        self.matrix = self.projection.as_matrix() * self.view.to_homogeneous();
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
    fn dimensions() {
        let mut c = Camera::default();
        let mat = c.matrix().clone();

        c.set_dimensions((1024, 768));
        assert_eq!(c.dimensions(), (1024, 768));
        assert_eq!(c.matrix(), &mat);

        c.set_dimensions((1400, 900));
        assert_eq!(c.dimensions(), (1400, 900));
        assert_ne!(c.matrix(), &mat);
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
    fn matrix() {
        let c = Camera::default();
        let m: &Matrix4<f32> = c.matrix();
        assert_eq!(m, &(c.projection.as_matrix() * c.view.to_homogeneous()));
    }
}
