use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, Vector3};
use std::{borrow::Borrow, f32};

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    matrix: Matrix4<f32>,
    projection: Perspective3<f32>,
    view: Isometry3<f32>,
}

impl Camera {
    pub fn new(
        dimensions: [u32; 2],
        fov_y: f32,
        frustum_z: [f32; 2],
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
    ) -> Self {
        let projection = Perspective3::new(
            dimensions[0] as f32 / dimensions[1] as f32,
            fov_y,
            frustum_z[0],
            frustum_z[1],
        );
        let view = Isometry3::look_at_rh(&eye, &target, &up);

        Camera {
            matrix: projection.as_matrix() * view.to_homogeneous(),
            projection,
            view,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new(
            [800, 600],
            f32::consts::PI / 4.0,
            [0.1, 1000.0],
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y(),
        )
    }
}

impl Borrow<Matrix4<f32>> for Camera {
    fn borrow(&self) -> &Matrix4<f32> {
        &self.matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _ = Camera::new(
            [800, 600],
            f32::consts::PI / 4.0,
            [0.1, 1000.0],
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
                [800, 600],
                f32::consts::PI / 4.0,
                [0.1, 1000.0],
                Point3::new(0.0, 0.0, 1.0),
                Point3::new(0.0, 0.0, -1.0),
                Vector3::y()
            )
        );
    }

    #[test]
    fn borrow() {
        let c = Camera::default();
        let m: &Matrix4<f32> = c.borrow();
        assert_eq!(m, &(c.projection.as_matrix() * c.view.to_homogeneous()));
    }
}
