use std::{f32, ops::Mul};

use affine_transform::AffineTransform;
use ecs::{Component, VecStorage};
use nalgebra::{Affine3, Isometry3, Matrix4, Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "AffineTransform<f32>", from = "AffineTransform<f32>")]
pub struct Model {
    model: Affine3<f32>,
    decomposed: AffineTransform<f32>,
}

impl Model {
    pub fn new(translation: Vector3<f32>, axisangle: Vector3<f32>, scale: Vector3<f32>) -> Self {
        let isometry = Isometry3::new(translation, axisangle);
        let scale_matrix = Affine3::from_matrix_unchecked(Matrix4::new(
            scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, scale.z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ));

        Model {
            model: isometry * scale_matrix,
            decomposed: AffineTransform::from_parts(isometry.translation, isometry.rotation, scale),
        }
    }

    pub fn look_at(eye: Point3<f32>, target: Point3<f32>, up: Vector3<f32>, scale: Vector3<f32>) -> Self {
        let isometry = Isometry3::look_at_rh(&eye, &target, &up);
        let scale_matrix = Affine3::from_matrix_unchecked(Matrix4::new(
            scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, scale.z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ));

        Model {
            model: isometry * scale_matrix,
            decomposed: AffineTransform::from_parts(isometry.translation, isometry.rotation, scale),
        }
    }

    pub fn identity() -> Self {
        Model {
            model: Affine3::identity(),
            decomposed: AffineTransform::identity(),
        }
    }

    pub fn transform_point(&self, point: &Point3<f32>) -> Point3<f32> {
        self.model.transform_point(point)
    }

    pub fn inverse_transform_point(&self, point: &Point3<f32>) -> Point3<f32> {
        self.model.inverse_transform_point(point)
    }

    pub fn set_position(&mut self, value: Point3<f32>) {
        self.decomposed.translation.vector = value.coords;
        self.recalculate_matrix();
    }

    pub fn set_orientation(&mut self, value: UnitQuaternion<f32>) {
        self.decomposed.rotation = value;
        self.recalculate_matrix();
    }

    pub fn set_scale(&mut self, value: Vector3<f32>) {
        self.decomposed.scale = value;
        self.recalculate_matrix();
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        self.model.matrix()
    }

    pub fn position(&self) -> Point3<f32> {
        Point3::from(self.decomposed.translation.vector)
    }

    pub fn orientation(&self) -> &UnitQuaternion<f32> {
        &self.decomposed.rotation
    }

    pub fn scale(&self) -> &Vector3<f32> {
        &self.decomposed.scale
    }

    fn recalculate_matrix(&mut self) {
        self.model = self.decomposed.recompose();
    }
}

impl Default for Model {
    fn default() -> Self {
        Model::identity()
    }
}

impl Component for Model {
    type Storage = VecStorage<Self>;
}

impl Mul<Model> for Model {
    type Output = Model;

    fn mul(self, rhs: Model) -> Model {
        &self * &rhs
    }
}

impl<'a, 'b> Mul<&'a Model> for &'b Model {
    type Output = Model;

    fn mul(self, rhs: &'a Model) -> Model {
        let product = self.model * rhs.model;

        Model {
            model: product,
            decomposed: product.into(),
        }
    }
}

impl From<AffineTransform<f32>> for Model {
    fn from(value: AffineTransform<f32>) -> Self {
        Model {
            model: value.recompose(),
            decomposed: value,
        }
    }
}

impl From<Model> for AffineTransform<f32> {
    fn from(value: Model) -> Self {
        value.decomposed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _: Model = Model::new(Vector3::y(), Vector3::z(), Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn default() {
        let _: Model = Default::default();
        assert_eq!(Model::default(), Model::identity());
    }

    #[test]
    fn identity() {
        let ident = Model::identity();
        let ident_mat: &Matrix4<f32> = ident.matrix();
        assert_eq!(ident_mat, &Matrix4::identity());
    }

    #[test]
    fn getters() {
        let ident = Model::identity();
        assert_eq!(ident.position(), Point3::new(0.0, 0.0, 0.0));
        assert_eq!(ident.orientation(), &UnitQuaternion::identity());
        assert_eq!(ident.scale(), &Vector3::new(1.0, 1.0, 1.0));

        let mat = Model::new(
            Vector3::new(2.0, 3.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 1.1, 1.0),
        );
        assert_eq!(mat.position(), Point3::new(2.0, 3.0, 1.0));
        assert_eq!(
            mat.orientation(),
            &UnitQuaternion::from_scaled_axis(Vector3::new(1.0, 0.0, 0.0))
        );
        assert_eq!(mat.scale(), &Vector3::new(1.0, 1.1, 1.0));
    }

    #[test]
    fn setters() {
        let mut ident = Model::identity();
        ident.set_position(Point3::new(1.0, 2.0, 3.0));
        assert_eq!(ident.position(), Point3::new(1.0, 2.0, 3.0));
        ident.set_orientation(UnitQuaternion::from_scaled_axis(Vector3::new(0.0, 1.5, 0.0)));
        assert_eq!(
            ident.orientation(),
            &UnitQuaternion::from_scaled_axis(Vector3::new(0.0, 1.5, 0.0))
        );
        ident.set_scale(Vector3::new(0.9, 0.4, 1.0));
        assert_eq!(ident.scale(), &Vector3::new(0.9, 0.4, 1.0));
    }

    #[test]
    fn multiply() {
        let a = Model::identity();
        let b = Model::identity();
        let expected = a.clone();

        assert_eq!(&a * &b, expected);
        assert_eq!(a * b, expected);
    }
}
