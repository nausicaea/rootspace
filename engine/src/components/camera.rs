use crate::geometry::ray::Ray;
use crate::components::model::Model;
use approx::ulps_eq;
use ecs::{Component, VecStorage};
use nalgebra::{Matrix4, Orthographic3, Perspective3, Point2, Point3, Unit, Vector3};
use serde::{Deserialize, Serialize};
use std::f32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Projection {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CameraSerDe {
    projection: Projection,
    dimensions: (u32, u32),
    fov_y: f32,
    frustum_z: (f32, f32),
    dpi_factor: f64,
}

impl From<Camera> for CameraSerDe {
    fn from(value: Camera) -> Self {
        CameraSerDe {
            projection: value.projection,
            dimensions: value.dimensions,
            fov_y: value.fov_y,
            frustum_z: value.frustum_z,
            dpi_factor: value.dpi_factor,
        }
    }
}

impl From<CameraSerDe> for Camera {
    fn from(value: CameraSerDe) -> Self {
        Camera::new(
            value.projection,
            value.dimensions,
            value.fov_y,
            value.frustum_z,
            value.dpi_factor,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "CameraSerDe", from = "CameraSerDe")]
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
    pub fn new<D: Into<Option<f64>>>(
        projection: Projection,
        dimensions: (u32, u32),
        fov_y: f32,
        frustum_z: (f32, f32),
        dpi_factor: D,
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

        Camera {
            projection,
            orthographic,
            perspective,
            dimensions,
            fov_y,
            frustum_z,
            dpi_factor: dpi_factor.into().unwrap_or(1.0),
        }
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
        Camera::new(
            Projection::Perspective,
            (800, 600),
            f32::consts::PI / 4.0,
            (0.1, 1000.0),
            1.0,
        )
    }
}

impl Component for Camera {
    type Storage = VecStorage<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _ = Camera::new(
            Projection::Perspective,
            (800, 600),
            f32::consts::PI / 4.0,
            (0.1, 1000.0),
            1.0,
        );
    }

    #[test]
    fn default() {
        assert_eq!(
            Camera::default(),
            Camera::new(
                Projection::Perspective,
                (800, 600),
                f32::consts::PI / 4.0,
                (0.1, 1000.0),
                1.0,
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
    fn word_vs_ndc() {
        let c = Camera::default();
        let m = Model::look_at(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y(), Vector3::new(1.0, 1.0, 1.0)
        );
        let p = Point3::new(1.0, 0.0, -5.0);
        let q = c.perspective.project_point(&m.transform_point(&p));
        let r = m.inverse_transform_point(&c.perspective.unproject_point(&q));

        assert_eq!(c.world_point_to_ndc(&m, &p), q);
        assert_eq!(c.ndc_point_to_world(&m, &q), r);
    }

    #[test]
    fn ui_vs_ndc() {
        let c = Camera::default();
        let p = Point3::new(1.0, 0.0, -5.0);
        let q = c.orthographic.project_point(&p);
        let r = c.orthographic.unproject_point(&q);

        assert_eq!(c.ui_point_to_ndc(&Point2::new(p.x, p.y), p.z), q);
        assert_eq!(c.ndc_point_to_ui(&q), (Point2::new(r.x, r.y), r.z));
    }

    #[test]
    fn world_vs_screen() {
        let c = Camera::default();
        let m = Model::look_at(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y(), Vector3::new(1.0, 1.0, 1.0)
        );
        let p = Point3::new(1.0, 0.0, -5.0);
        let q = {
            let tmp = c.world_point_to_ndc(&m, &p);
            let w = c.dimensions.0 as f32;
            let h = c.dimensions.1 as f32;
            Point2::new(
                ((w / 2.0) * (tmp.x + 1.0)).round() as u32,
                ((h / 2.0) * (1.0 - tmp.y)).round() as u32,
            )
        };
        let r = {
            let w = c.dimensions.0 as f32;
            let h = c.dimensions.1 as f32;
            let tmp = Point3::new((2.0 * q.x as f32) / w - 1.0, 1.0 - (2.0 * q.y as f32) / h, 1.0);
            let origin = -m.position();
            let target = c.ndc_point_to_world(&m, &tmp).coords;

            Unit::try_new(target, f32::EPSILON).map(|direction| Ray { origin, direction })
        };

        assert_eq!(c.world_point_to_screen(&m, &p), q);
        assert_eq!(c.screen_point_to_world_ray(&m, &q), r);
    }

    #[test]
    fn ui_vs_screen() {
        let c = Camera::default();
        let p = Point3::new(1.0, 0.0, -5.0);
        let q = {
            let tmp = c.ui_point_to_ndc(&Point2::new(p.x, p.y), p.z);
            let w = c.dimensions.0 as f32;
            let h = c.dimensions.1 as f32;
            Point2::new(
                ((w / 2.0) * (tmp.x + 1.0)).round() as u32,
                ((h / 2.0) * (1.0 - tmp.y)).round() as u32,
            )
        };
        let r = {
            let w = c.dimensions.0 as f32;
            let h = c.dimensions.1 as f32;
            let tmp = Point3::new((2.0 * q.x as f32) / w - 1.0, 1.0 - (2.0 * q.y as f32) / h, 1.0);
            let origin = Point3::new(0.0, 0.0, 0.0);
            let target = {
                let (t, d) = c.ndc_point_to_ui(&tmp);
                Vector3::new(t.x, t.y, d)
            };

            Unit::try_new(target, f32::EPSILON).map(|direction| Ray { origin, direction })
        };

        assert_eq!(c.ui_point_to_screen(&Point2::new(p.x, p.y), p.z), q);
        assert_eq!(c.screen_point_to_ui_ray(&q), r);
    }

    #[test]
    fn world_matrix_accessor() {
        let c = Camera::default();
        let m: &Matrix4<f32> = c.world_matrix();
        assert_eq!(m, c.perspective.as_matrix());
    }

    #[test]
    fn ui_matrix_accessor() {
        let c = Camera::default();
        let m: &Matrix4<f32> = c.ui_matrix();
        assert_eq!(m, c.orthographic.as_matrix());
    }
}
