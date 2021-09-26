use std::{f32, ops::Mul};

use affine_transform::AffineTransform;
use ecs::{Component, VecStorage};
use nalgebra::{Affine3, Matrix4, Point3, Translation3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::iter::Product;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "AffineTransform<f32>", from = "AffineTransform<f32>")]
pub struct Model {
    model: Affine3<f32>,
    decomposed: AffineTransform<f32>,
}

impl Model {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::default()
    }

    pub fn transform_point(&self, point: &Point3<f32>) -> Point3<f32> {
        self.model.transform_point(point)
    }

    pub fn inverse_transform_point(&self, point: &Point3<f32>) -> Point3<f32> {
        self.model.inverse_transform_point(point)
    }

    pub fn set_position(&mut self, value: Point3<f32>) {
        self.decomposed.translation = Translation3::new(value.x, value.y, value.z);
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
        Model::builder().build()
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
        let product = &self.model * &rhs.model;

        Model {
            model: product,
            decomposed: product.into(),
        }
    }
}

impl<'a> Product<&'a Model> for Model {
    fn product<I: Iterator<Item = &'a Model>>(iter: I) -> Self {
        iter.fold(Model::default(), |state, value| &state * value)
    }
}

impl Product for Model {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Model::default(), |state, value| state * value)
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

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "position: [{}, {}, {}], orientation: {}, scale: [{}, {}, {}]",
            self.position().x,
            self.position().y,
            self.position().z,
            self.orientation(),
            self.scale().x,
            self.scale().y,
            self.scale().z
        )
    }
}

#[derive(Debug)]
pub struct ModelBuilder {
    position: Point3<f32>,
    orientation: UnitQuaternion<f32>,
    scale: Vector3<f32>,
}

impl ModelBuilder {
    pub fn with_position(mut self, position: Point3<f32>) -> Self {
        self.position = position;
        self
    }

    pub fn with_orientation(mut self, orientation: UnitQuaternion<f32>) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_scale(mut self, scale: Vector3<f32>) -> Self {
        self.scale = scale;
        self
    }

    pub fn build(self) -> Model {
        let at = AffineTransform::from_parts(
            Translation3::new(self.position.x, self.position.y, self.position.z),
            self.orientation,
            self.scale,
        );

        Model {
            model: at.recompose(),
            decomposed: at,
        }
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        ModelBuilder {
            position: Point3::from([0.0; 3]),
            orientation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let _: Model = Default::default();
    }

    #[test]
    fn getters() {
        let ident = Model::default();
        assert_eq!(ident.position(), Point3::new(0.0, 0.0, 0.0));
        assert_eq!(ident.orientation(), &UnitQuaternion::identity());
        assert_eq!(ident.scale(), &Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn setters() {
        let mut ident = Model::default();
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
        let a = Model::default();
        let b = Model::default();
        let expected = a.clone();

        assert_eq!(&a * &b, expected);
        assert_eq!(a * b, expected);
    }
}
