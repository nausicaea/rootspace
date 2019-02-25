use ecs::{Component, VecStorage};
use alga::linear::{ProjectiveTransformation, Transformation};
use crate::geometry::ray::Ray;
use nalgebra::{Isometry3, Matrix4, Orthographic3, Perspective3, Point2, Point3, Unit, Vector3};
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

    pub fn set_dimensions(&mut self, value: (u32, u32)) {
        if value != self.dimensions {
            self.perspective.set_aspect(value.0 as f32 / value.1 as f32);
            self.dimensions = value;
            self.recalculate_matrices();
        }
    }

    pub fn set_dpi_factor(&mut self, value: f64) {
        if value != self.dpi_factor {
            self.dpi_factor = value;
            self.recalculate_matrices();
        }
    }

    pub fn world_matrix(&self) -> &Matrix4<f32> {
        &self.world_matrix
    }

    pub fn ui_matrix(&self) -> &Matrix4<f32> {
        &self.ui_matrix
    }

    pub fn position(&self) -> Point3<f32> {
        Point3::from(self.view.translation.vector)
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
    pub fn world_point_to_ndc(&self, point: &Point3<f32>) -> Point3<f32> {
        self.perspective.project_point(&self.view.transform_point(point))
    }

    /// Transforms a point in normalized device coordinates to world-space.
    pub fn ndc_point_to_world(&self, point: &Point3<f32>) -> Point3<f32> {
        self.view
            .inverse_transform_point(&self.perspective.unproject_point(point))
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
    pub fn world_point_to_screen(&self, point: &Point3<f32>) -> Point2<u32> {
        self.ndc_point_to_screen(&self.world_point_to_ndc(point))
    }

    /// Transforms a screen point to world-space as a ray originating from the camera.
    pub fn screen_point_to_world_ray(&self, point: &Point2<u32>) -> Option<Ray<f32>> {
        let origin = Point3::from(-self.view.translation.vector);
        let target = self.screen_point_to_world(point).coords;
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
    fn screen_point_to_world(&self, point: &Point2<u32>) -> Point3<f32> {
        self.ndc_point_to_world(&self.screen_point_to_ndc(point))
    }

    /// Transforms a point in screen space to ui space.
    fn screen_point_to_ui(&self, point: &Point2<u32>) -> (Point2<f32>, f32) {
        self.ndc_point_to_ui(&self.screen_point_to_ndc(point))
    }

    /// Updates the cached matrices to speed up the rendering calls.
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

impl Component for Camera {
    type Storage = VecStorage<Self>;
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
    fn word_vs_ndc() {
        let c = Camera::default();
        let p = Point3::new(1.0, 0.0, -5.0);
        let q = c.perspective.project_point(&c.view.transform_point(&p));
        let r = c.view.inverse_transform_point(&c.perspective.unproject_point(&q));

        assert_eq!(c.world_point_to_ndc(&p), q);
        assert_eq!(c.ndc_point_to_world(&q), r);
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
        let p = Point3::new(1.0, 0.0, -5.0);
        let q = {
            let tmp = c.world_point_to_ndc(&p);
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
            let origin = Point3::from(-c.view.translation.vector);
            let target = c.ndc_point_to_world(&tmp).coords;

            Unit::try_new(target, f32::EPSILON)
                .map(|direction| Ray { origin, direction })
        };

        assert_eq!(c.world_point_to_screen(&p), q);
        assert_eq!(c.screen_point_to_world_ray(&q), r);
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

            Unit::try_new(target, f32::EPSILON)
                .map(|direction| Ray { origin, direction })
        };

        assert_eq!(c.ui_point_to_screen(&Point2::new(p.x, p.y), p.z), q);
        assert_eq!(c.screen_point_to_ui_ray(&q), r);
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
