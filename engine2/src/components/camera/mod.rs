mod camera_builder;
pub mod projection;

use approx::ulps_eq;
use ecs::{Component, VecStorage};
use glamour::{Mat4, Ortho, Persp};

use self::{camera_builder::CameraBuilder, projection::Projection};

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    projection: Projection,
    ortho: Ortho<f32>,
    persp: Persp<f32>,
    dimensions: (u32, u32),
    fov_y: f32,
    frustum_z: (f32, f32),
    dpi_factor: f64,
}

impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    pub fn set_dimensions(&mut self, value: (u32, u32)) {
        if value == self.dimensions {
            return;
        }

        self.dimensions = value;
        self.rebuild_projections();
    }

    pub fn set_dpi_factor(&mut self, value: f64) {
        if ulps_eq!(value, self.dpi_factor) {
            return;
        }
        self.dpi_factor = value;
    }

    pub fn as_world_matrix(&self) -> &Mat4<f32> {
        match self.projection {
            Projection::Perspective => self.persp.as_matrix(),
            Projection::Orthographic => self.ortho.as_matrix(),
        }
    }

    pub fn as_ui_matrix(&self) -> &Mat4<f32> {
        self.ortho.as_matrix()
    }

    pub fn projection(&self) -> Projection {
        self.projection
    }

    pub fn fov_y(&self) -> f32 {
        self.fov_y
    }

    pub fn frustum_z(&self) -> (f32, f32) {
        self.frustum_z
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

    pub fn dpi_factor(&self) -> f64 {
        self.dpi_factor
    }

    fn rebuild_projections(&mut self) {
        self.ortho = Ortho::builder()
            .with_aspect(self.dimensions.0 as f32 / self.dimensions.1 as f32)
            .with_fov_y(self.fov_y)
            .with_near_z(self.frustum_z.0)
            .with_far_z(self.frustum_z.1)
            .build()
            .unwrap_or_else(|e| panic!("cannot update the orthographic projection matrix: {}", e));
        self.persp = Persp::builder()
            .with_aspect(self.dimensions.0 as f32 / self.dimensions.1 as f32)
            .with_fov_y(self.fov_y)
            .with_near_z(self.frustum_z.0)
            .with_far_z(self.frustum_z.1)
            .build()
            .unwrap_or_else(|e| panic!("cannot update the perspective projection matrix: {}", e));
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::builder().build()
    }
}

impl Component for Camera {
    type Storage = VecStorage<Self>;
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use proptest::{num::f64::NORMAL, prelude::*};

    use super::*;

    #[test]
    fn implements_default() {
        let _: Camera = Default::default();
    }

    #[test]
    fn blank_builder_is_the_same_as_default() {
        let ca: Camera = Camera::builder().build();
        let cb: Camera = Default::default();

        assert_eq!(ca, cb);
    }

    #[test]
    fn provides_dimensions_accessor_with_defaults() {
        let c = Camera::default();
        assert_eq!(c.dimensions(), (800, 600));
    }

    #[test]
    fn provides_dimensions_accessor() {
        let c = Camera::builder().with_dimensions((1320, 1024)).build();
        assert_eq!(c.dimensions(), (1320, 1024));
    }

    #[test]
    #[ignore]
    fn provides_ui_matrix_accessor_with_defaults() {
        let c = Camera::default();
        assert_ulps_eq!(
            c.as_ui_matrix(),
            &Mat4::from([
                [0.0025f32, 0.0f32, 0.0f32, 0.0f32],
                [0.0f32, 0.00333333f32, 0.0f32, 0.0f32],
                [0.0f32, 0.0f32, -0.00200020f32, 0.0f32],
                [0.0f32, 0.0f32, -1.00020002f32, 1.0f32],
            ])
        );
    }

    proptest! {
        #[test]
        fn dimensions_may_be_changed(num: (u32, u32)) {
            let mut c = Camera::default();
            c.set_dimensions(num);
            prop_assert_eq!(c.dimensions(), num);
        }

        #[test]
        fn dpi_factor_may_be_changed(num in NORMAL) {
            let mut c = Camera::default();
            c.set_dpi_factor(num);

            prop_assert_eq!(c.dpi_factor(), num);
        }
    }
}
