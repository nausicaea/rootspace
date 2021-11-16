pub mod camera_builder;
mod camera_ser_de;
pub mod projection;

use self::camera_builder::CameraBuilder;
use self::camera_ser_de::CameraSerDe;
use self::projection::Projection;
use approx::ulps_eq;
use ecs::{Component, VecStorage};
use nalgebra::{Matrix4, Orthographic3, Perspective3, Point2, Point3, Unit, Vector3};
use serde::{Deserialize, Serialize};

use crate::{components::model::Model, geometry::ray::Ray};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "self::camera_ser_de::CameraSerDe", from = "self::camera_ser_de::CameraSerDe")]
pub struct Camera {
    projection: Projection,
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

    /// Transforms a point in world-space to normalized device coordinates.
    pub fn world_point_to_ndc(&self, model: &Model, point: &Point3<f32>) -> Point3<f32> {
        match self.projection {
            Projection::Perspective => self.perspective.project_point(&model.transform_point(point)),
            Projection::Orthographic => self.orthographic.project_point(&model.transform_point(point)),
        }
    }

    /// Transforms a point in normalized device coordinates to world-space.
    pub fn ndc_point_to_world(&self, model: &Model, point: &Point3<f32>) -> Point3<f32> {
        let cam_space = match self.projection {
            Projection::Perspective => self.perspective.unproject_point(point),
            Projection::Orthographic => self.orthographic.unproject_point(point),
        };

        model.inverse_transform_point(&cam_space)
    }

    /// Transforms a point in ui-space to normalized device coordinates.
    pub fn ui_point_to_ndc(&self, point: &Point2<f32>, depth: f32) -> Point3<f32> {
        self.orthographic.project_point(&Point3::new(point.x, point.y, depth))
    }

    /// Transforms a point in normalized device coordinates to ui-space.
    pub fn ndc_point_to_ui(&self, point: &Point3<f32>) -> (Point2<f32>, f32) {
        let p = self.orthographic.unproject_point(point);
        (Point2::new(p.x, p.y), p.z)
    }

    /// Transforms a point in world-space to a screen point.
    pub fn world_point_to_screen(&self, model: &Model, point: &Point3<f32>) -> Point2<u32> {
        self.ndc_point_to_screen(&self.world_point_to_ndc(model, point))
    }

    /// Transforms a screen point to world-space as a ray originating from the camera.
    pub fn screen_point_to_world_ray(&self, model: &Model, point: &Point2<u32>) -> Option<Ray<f32>> {
        let origin = -model.position();
        let target = self.screen_point_to_world(model, point).coords;
        Unit::try_new(target, f32::EPSILON).map(|direction| Ray { origin, direction })
    }

    pub fn ui_point_to_screen(&self, point: &Point2<f32>, depth: f32) -> Point2<u32> {
        self.ndc_point_to_screen(&self.ui_point_to_ndc(point, depth))
    }

    /// Transforms a screen point to ui-space as a ray originating from the camera.
    pub fn screen_point_to_ui_ray(&self, point: &Point2<u32>) -> Option<Ray<f32>> {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let target = {
            let (t, d) = self.screen_point_to_ui(point);
            Vector3::new(t.x, t.y, d)
        };
        Unit::try_new(target, f32::EPSILON).map(|direction| Ray { origin, direction })
    }

    /// Transforms a point in normalized device coordinates to screen-space.
    fn ndc_point_to_screen(&self, point: &Point3<f32>) -> Point2<u32> {
        let w = self.dimensions.0 as f32;
        let h = self.dimensions.1 as f32;

        Point2::new(
            ((w / 2.0) * (point.x + 1.0)).round() as u32,
            ((h / 2.0) * (1.0 - point.y)).round() as u32,
        )
    }

    /// Projects a point in screen space to the far plane of the normalized device coordinate cube.
    /// Note that this assumes NDC to be a left-handed coordinate system.
    fn screen_point_to_ndc(&self, point: &Point2<u32>) -> Point3<f32> {
        let w = self.dimensions.0 as f32;
        let h = self.dimensions.1 as f32;

        Point3::new((2.0 * point.x as f32) / w - 1.0, 1.0 - (2.0 * point.y as f32) / h, 1.0)
    }

    /// Transforms a point in screen space to world space.
    fn screen_point_to_world(&self, model: &Model, point: &Point2<u32>) -> Point3<f32> {
        self.ndc_point_to_world(model, &self.screen_point_to_ndc(point))
    }

    /// Transforms a point in screen space to ui space.
    fn screen_point_to_ui(&self, point: &Point2<u32>) -> (Point2<f32>, f32) {
        self.ndc_point_to_ui(&self.screen_point_to_ndc(point))
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
    use approx::{assert_ulps_eq, ulps_eq};
    use nalgebra::Vector4;

    use super::*;
    use proptest::prelude::*;
    use std::convert::TryFrom;

    #[test]
    fn implements_default() {
        let _: Camera = Default::default();
    }

    #[test]
    fn provides_builder() {
        let _: CameraBuilder = Camera::builder();
    }

    #[test]
    fn builder_provides_with_dimensions_method() {
        let _ = Camera::builder().with_dimensions((1, 1));
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
        fn ui_ndc_projection_is_the_same_as_matrix_multiplication(fovy in 0usize..1000, fnz: f32, fdist in 1usize..1000, x: f32, y: f32, d: f32) {
            prop_assume!(fnz.is_normal());
            let fovy = (fovy as f32 / 1000.0f32) * std::f32::consts::PI;
            let ffz = fnz + fdist as f32;
            prop_assume!(ffz - fnz > f32::EPSILON);

            let c = Camera::builder()
                .with_projection(Projection::Orthographic)
                .with_dimensions((800, 600))
                .with_dpi_factor(1.0)
                .with_fov_y(fovy)
                .with_frustum_z((fnz, ffz))
                .build();
            let p = Point2::new(x, y);
            let pproj: Point3<f32> = c.ui_point_to_ndc(&p, d);

            let mmul: Vector4<f32> = c.ui_matrix() * Vector4::new(x, y, d, 1.0f32);
            let mmul: Vector4<f32> = mmul / mmul.w;

            let (tx, ty, tz) = (pproj.x, pproj.y, pproj.z);
            let (mx, my, mz) = (mmul.x, mmul.y, mmul.z);
            prop_assert!(
                ((tx.is_nan() && mx.is_nan()) || (!tx.is_nan() && (tx == mx))) &&
                ((ty.is_nan() && my.is_nan()) || (!ty.is_nan() && (ty == my))) &&
                ((tz.is_nan() && mz.is_nan()) || (!tz.is_nan() && (tz == mz))),
                "{:?} != {:?}", pproj, mmul
            );
        }

        #[test]
        fn world_ndc_projection_is_the_same_as_matrix_multiplication(fovy in 0usize..1000, fnz: f32, fdist in 1usize..1000, x: f32, y: f32, z: f32) {
            prop_assume!(fnz.is_normal());
            let fovy = (fovy as f32 / 1000.0f32) * std::f32::consts::PI;
            let ffz = fnz + fdist as f32;
            prop_assume!(ffz - fnz > f32::EPSILON);

            let c = Camera::builder()
                .with_projection(Projection::Perspective)
                .with_dimensions((800, 600))
                .with_dpi_factor(1.0)
                .with_fov_y(fovy)
                .with_frustum_z((fnz, ffz))
                .build();
            let cam_mdl: Model = Model::default();
            let p = Point3::new(x, y, z);
            let pproj: Point3<f32> = c.world_point_to_ndc(&cam_mdl, &p);

            let mmul: Vector4<f32> = c.world_matrix() * cam_mdl.matrix() * Vector4::new(x, y, z, 1.0f32);
            let mmul: Vector4<f32> = mmul / mmul.w;

            let (tx, ty, tz) = (pproj.x, pproj.y, pproj.z);
            let (mx, my, mz) = (mmul.x, mmul.y, mmul.z);
            prop_assert!(
                ((tx.is_nan() && mx.is_nan()) || (!tx.is_nan() && (tx == mx))) &&
                ((ty.is_nan() && my.is_nan()) || (!ty.is_nan() && (ty == my))) &&
                ((tz.is_nan() && mz.is_nan()) || (!tz.is_nan() && (tz == mz))),
                "{:?} != {:?}", pproj, mmul
            );
        }
    }
}
