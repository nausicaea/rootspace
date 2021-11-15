use crate::components::{Camera, Projection};
use nalgebra::{Orthographic3, Perspective3};

#[derive(Debug)]
pub struct CameraBuilder {
    projection: Projection,
    dimensions: (u32, u32),
    fov_y: f32,
    frustum_z: (f32, f32),
    dpi_factor: f64,
}

impl CameraBuilder {
    pub fn with_projection(mut self, projection: Projection) -> Self {
        self.projection = projection;
        self
    }

    pub fn with_dimensions(mut self, dimensions: (u32, u32)) -> Self {
        self.dimensions = dimensions;
        self
    }

    pub fn with_fov_y(mut self, fov_y: f32) -> Self {
        self.fov_y = fov_y;
        self
    }

    pub fn with_frustum_z(mut self, frustum_z: (f32, f32)) -> Self {
        assert_ne!(
            frustum_z.0, frustum_z.1,
            "The near-plane and far-plane must not be superimposed."
        );
        self.frustum_z = frustum_z;
        self
    }

    pub fn with_dpi_factor(mut self, dpi_factor: f64) -> Self {
        self.dpi_factor = dpi_factor;
        self
    }

    pub fn build(self) -> Camera {
        let orthographic = Orthographic3::new(
            self.dimensions.0 as f32 / -2.0,
            self.dimensions.0 as f32 / 2.0,
            self.dimensions.1 as f32 / -2.0,
            self.dimensions.1 as f32 / 2.0,
            self.frustum_z.0,
            self.frustum_z.1,
        );
        let perspective = Perspective3::new(
            self.dimensions.0 as f32 / self.dimensions.1 as f32,
            self.fov_y,
            self.frustum_z.0,
            self.frustum_z.1,
        );

        Camera {
            projection: self.projection,
            orthographic,
            perspective,
            dimensions: self.dimensions,
            fov_y: self.fov_y,
            frustum_z: self.frustum_z,
            dpi_factor: self.dpi_factor,
        }
    }
}

impl Default for CameraBuilder {
    fn default() -> Self {
        CameraBuilder {
            projection: Projection::Perspective,
            dimensions: (800, 600),
            fov_y: std::f32::consts::PI / 4.0,
            frustum_z: (0.1, 1000.0),
            dpi_factor: 1.0,
        }
    }
}
