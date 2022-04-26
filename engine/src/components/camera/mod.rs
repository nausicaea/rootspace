pub mod camera_builder;
mod camera_ser_de;
pub mod projection;

use approx::ulps_eq;
use ecs::{Component, VecStorage};
use nalgebra::{Matrix4, Orthographic3, Perspective3, Point2, Point3, Unit, Vector3};
use glamour::{Vec4, Vec2, Ray, Ortho, Persp};
use serde::{Deserialize, Serialize};

use self::{camera_builder::CameraBuilder, camera_ser_de::CameraSerDe, projection::Projection};
use crate::components::model::Model;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "self::camera_ser_de::CameraSerDe", from = "self::camera_ser_de::CameraSerDe")]
pub struct Camera {
    projection: Projection,
    ortho: Ortho<f32>,
    persp: Persp<f32>,
    orthographic: Orthographic3<f32>,
    perspective: Perspective3<f32>,
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
        if value.0 != self.dimensions.0 {
            self.orthographic
                .set_left_and_right(value.0 as f32 / -2.0, value.0 as f32 / 2.0);
        }
        if value.1 != self.dimensions.1 {
            self.orthographic
                .set_bottom_and_top(value.1 as f32 / -2.0, value.1 as f32 / 2.0);
        }
        self.perspective.set_aspect(value.0 as f32 / value.1 as f32);
        self.dimensions = value;
    }

    pub fn set_dpi_factor(&mut self, value: f64) {
        if ulps_eq!(value, self.dpi_factor) {
            return;
        }
        self.dpi_factor = value;
    }

    pub fn set_fov_y(&mut self, value: f32) {
        if ulps_eq!(value, self.fov_y) {
            return;
        }
        self.perspective.set_fovy(value);
        self.fov_y = value;
    }

    pub fn set_frustum_z(&mut self, value: (f32, f32)) {
        if ulps_eq!(value.0, self.frustum_z.0) && ulps_eq!(value.1, self.frustum_z.1) {
            return;
        }
        self.orthographic.set_znear_and_zfar(value.0, value.1);
        self.perspective.set_znear_and_zfar(value.0, value.1);
        self.frustum_z = value;
    }

    pub fn world_matrix(&self) -> &Matrix4<f32> {
        match self.projection {
            Projection::Perspective => self.perspective.as_matrix(),
            Projection::Orthographic => self.orthographic.as_matrix(),
        }
    }

    pub fn ui_matrix(&self) -> &Matrix4<f32> {
        self.orthographic.as_matrix()
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

    /// Transforms a point or vector in world-space to normalized device coordinates.
    pub fn world_to_ndc(&self, model: &Model, v: &Vec4<f32>) -> Vec4<f32> {
        match self.projection {
            Projection::Perspective => &self.persp * model * v,
            Projection::Orthographic => &self.ortho * model * v,
        }
    }

    /// Transforms a point or vector in normalized device coordinates to world-space.
    pub fn ndc_to_world(&self, model: &Model, v: &Vec4<f32>) -> Vec4<f32> {
        match self.projection {
            Projection::Perspective => &model.inv() * &self.persp.inv() * v,
            Projection::Orthographic => &model.inv() * &self.ortho.inv() * v,
        }
    }

    /// Transforms a point or vector in ui-space to normalized device coordinates.
    pub fn ui_to_ndc(&self, v: &Vec2<f32>, depth: f32, w: f32) -> Vec4<f32> {
        &self.ortho * &Vec4::new(v.x(), v.y(), depth, w)
    }

    /// Transforms a point or vector in normalized device coordinates to ui-space.
    pub fn ndc_to_ui(&self, v: &Vec4<f32>) -> (Vec2<f32>, f32, f32) {
        let v: Vec4<f32> = &self.ortho.inv() * v;
        (Vec2::new(v.x(), v.y()), v.z(), v.w())
    }

    /// Transforms a point or vector in world-space to a screen point.
    pub fn world_to_screen(&self, model: &Model, v: &Vec4<f32>) -> Vec2<u32> {
        self.ndc_to_screen(&self.world_to_ndc(v))
    }

    /// Transforms a screen point or vector to world-space as a ray originating from the camera.
    pub fn screen_to_world_ray(&self, model: &Model, v: &Vec2<u32>) -> Option<Ray<f32>> {
        let origin = -model.translation();
        let target = self.screen_to_world(model, v);
        Unit::try_new(target, f32::EPSILON).map(|direction| Ray { origin, direction })
    }

    pub fn ui_to_screen(&self, point: &Point2<f32>, depth: f32) -> Point2<u32> {
        self.ndc_to_screen(&self.ui_to_ndc(point, depth))
    }

    /// Transforms a screen point or vector to ui-space as a ray originating from the camera.
    pub fn screen_to_ui_ray(&self, point: &Point2<u32>) -> Option<Ray<f32>> {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let target = {
            let (t, d) = self.screen_point_to_ui(point);
            Vector3::new(t.x, t.y, d)
        };
        Unit::try_new(target, f32::EPSILON).map(|direction| Ray { origin, direction })
    }

    /// Transforms a point or vector in normalized device coordinates to screen-space.
    fn ndc_to_screen(&self, v: &Vec4<f32>) -> Vec2<u32> {
        let w = self.dimensions.0 as f32;
        let h = self.dimensions.1 as f32;

        Vec2::new(
            ((w / 2.0) * (v.x() + 1.0)).round() as u32,
            ((h / 2.0) * (1.0 - v.y())).round() as u32,
        )
    }

    /// Projects a point or vector in screen space to the far plane of the normalized device coordinate cube.
    /// Note that this assumes NDC to be a left-handed coordinate system.
    fn screen_to_ndc(&self, v: &Vec2<u32>) -> Vec4<f32> {
        let w = self.dimensions.0 as f32;
        let h = self.dimensions.1 as f32;

        Vec4::new(
            (2.0 * v.x() as f32) / w - 1.0,
            1.0 - (2.0 * v.y() as f32) / h, 
            1.0,
            1.0,
        )
    }

    /// Transforms a point or vector in screen space to world space.
    fn screen_to_world(&self, model: &Model, v: &Vec2<u32>) -> Vec4<f32> {
        self.ndc_to_world(model, &self.screen_to_ndc(v))
    }

    /// Transforms a point or vector in screen space to ui space.
    fn screen_to_ui(&self, v: &Vec2<u32>) -> (Vec2<f32>, f32) {
        self.ndc_to_ui(&self.screen_to_ndc(v))
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

impl From<CameraSerDe> for Camera {
    fn from(value: CameraSerDe) -> Self {
        Camera::builder()
            .with_projection(value.projection)
            .with_dimensions(value.dimensions)
            .with_fov_y(value.fov_y)
            .with_frustum_z(value.frustum_z)
            .with_dpi_factor(value.dpi_factor)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use approx::{assert_ulps_eq, ulps_eq};
    use nalgebra::Vector4;
    use proptest::prelude::*;

    use super::*;
    use crate::utilities::validate_float;

    #[test]
    fn implements_default() {
        let _: Camera = Default::default();
    }

    #[test]
    fn provides_builder() {
        let _: CameraBuilder = Camera::builder();
    }

    #[test]
    fn blank_builder_is_the_same_as_default() {
        let ca: Camera = Camera::builder().build();
        let cb: Camera = Default::default();

        assert_eq!(ca, cb);
        // TODO: assert_ulps_eq!(ca, cb);
    }

    #[test]
    fn builder_accepts_dimensions() {
        let _: CameraBuilder = Camera::builder().with_dimensions((1, 1));
    }

    #[test]
    fn builder_accepts_projection() {
        let _: CameraBuilder = CameraBuilder::default().with_projection(Projection::Orthographic);
    }

    #[test]
    fn builder_accepts_field_of_view() {
        let _: CameraBuilder = CameraBuilder::default().with_fov_y(1.0);
    }

    #[test]
    fn builder_accepts_frustum_limits() {
        let _: CameraBuilder = CameraBuilder::default().with_frustum_z((0.1, 1020.0));
    }

    #[test]
    fn builder_accepts_dpi_factor() {
        let _: CameraBuilder = CameraBuilder::default().with_dpi_factor(2.0);
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
    fn provides_ui_matrix_accessor_with_defaults() {
        let c = Camera::default();
        assert_ulps_eq!(
            c.ui_matrix(),
            &Matrix4::try_from([
                [0.0025f32, 0.0f32, 0.0f32, 0.0f32],
                [0.0f32, 0.00333333f32, 0.0f32, 0.0f32],
                [0.0f32, 0.0f32, -0.00200020f32, 0.0f32],
                [0.0f32, 0.0f32, -1.00020002f32, 1.0f32],
            ])
            .unwrap()
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
        fn dpi_factor_may_be_changed(num: f64) {
            let mut c = Camera::default();
            c.set_dpi_factor(num);

            if !validate_float(&[num]) {
                return Ok(())
            } else {
                prop_assert_eq!(c.dpi_factor(), num);
            }
        }
    }
}
