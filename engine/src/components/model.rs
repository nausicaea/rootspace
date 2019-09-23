use ecs::{Component, VecStorage};

use affine_transform::AffineTransform;
use nalgebra::{Affine3, Isometry3, Matrix4, Point3, UnitQuaternion, Vector3};
use serde::{Serialize, Deserialize};
use std::{f32, ops::Mul};
use typename::TypeName;

#[derive(Debug, Clone, PartialEq, TypeName, Serialize, Deserialize)]
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

    pub fn identity() -> Self {
        Model {
            model: Affine3::identity(),
            decomposed: AffineTransform::identity(),
        }
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

// impl DepthOrderingTrait for Model {
//     fn depth_index(&self) -> i32 {
//         (self.decomposed.translation.vector.z / f32::EPSILON).round() as i32
//     }
// }

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
    fn default() {
        assert_eq!(Model::default(), Model::identity());
    }

    // #[test]
    // fn depth_ordering() {
    //     let a = Model::new(
    //         Vector3::new(-1.0, 0.0, -10.35),
    //         Vector3::new(0.0, 0.0, 0.0),
    //         Vector3::new(1.0, 1.0, 1.0),
    //     );
    //     let b = Model::new(
    //         Vector3::new(-1.0, 0.0, 0.0),
    //         Vector3::new(0.0, 0.0, 0.0),
    //         Vector3::new(1.0, 1.0, 1.0),
    //     );
    //     let c = Model::new(
    //         Vector3::new(-1.0, 0.0, 12.35),
    //         Vector3::new(0.0, 0.0, 0.0),
    //         Vector3::new(1.0, 1.0, 1.0),
    //     );

    //     let a_idx = a.depth_index();
    //     let b_idx = b.depth_index();
    //     let c_idx = c.depth_index();

    //     assert!(a_idx < b_idx);
    //     assert!(b_idx < c_idx);
    // }

    #[test]
    fn multiply() {
        let a = Model::identity();
        let b = Model::identity();
        let expected = a.clone();

        assert_eq!(&a * &b, expected);
        assert_eq!(a * b, expected);
    }
}
