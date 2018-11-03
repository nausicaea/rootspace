use super::{DepthOrderingTrait, TransformTrait, camera::Camera};
use affine_transform::AffineTransform;
use nalgebra::{Affine3, Isometry3, Matrix4, Vector3};
use std::{borrow::Borrow, f32};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    World,
    Ndc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    layer: Layer,
    model: Affine3<f32>,
    decomposed: AffineTransform<f32>,
}

impl Model {
    pub fn new(layer: Layer, translation: Vector3<f32>, axisangle: Vector3<f32>, scale: Vector3<f32>) -> Self {
        let isometry = Isometry3::new(translation, axisangle);
        let scale_matrix = Affine3::from_matrix_unchecked(Matrix4::new(
            scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, scale.z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ));

        Model {
            layer,
            model: isometry * scale_matrix,
            decomposed: AffineTransform::from_parts(isometry.translation, isometry.rotation, scale),
        }
    }

    pub fn identity(layer: Layer) -> Self {
        Model {
            layer,
            model: Affine3::identity(),
            decomposed: AffineTransform::identity(),
        }
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        self.model.matrix()
    }
}

impl Default for Model {
    fn default() -> Self {
        Model::identity(Layer::World)
    }
}

impl DepthOrderingTrait for Model {
    fn depth_index(&self) -> i32 {
        (self.decomposed.translation.vector.z / f32::EPSILON).round() as i32
    }
}

impl TransformTrait for Model {
    type Camera = Camera;

    fn transform(&self, camera: &Camera, rhs: &Model) -> Option<Model> {
        if self.layer == rhs.layer {
            let product = self.model * rhs.model;

            Some(Model {
                layer: rhs.layer,
                model: product,
                decomposed: product.into(),
            })
        } else if self.layer == Layer::World && rhs.layer == Layer::Ndc {
            let projected = camera.matrix() * self.matrix();
            let product = Affine3::from_matrix_unchecked(projected) * rhs.model;

            Some(Model {
                layer: rhs.layer,
                model: product,
                decomposed: product.into(),
            })
        } else {
            None
        }
    }
}

impl Borrow<Matrix4<f32>> for Model {
    fn borrow(&self) -> &Matrix4<f32> {
        self.model.matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _: Model = Model::new(Layer::World, Vector3::y(), Vector3::z(), Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn identity() {
        let ident = Model::identity(Layer::World);
        let ident_mat: &Matrix4<f32> = ident.borrow();
        assert_eq!(ident_mat, &Matrix4::identity());
    }

    #[test]
    fn default() {
        assert_eq!(Model::default(), Model::identity(Layer::World));
    }

    #[test]
    fn depth_ordering() {
        let a = Model::new(
            Layer::World,
            Vector3::new(-1.0, 0.0, -10.35),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        let b = Model::new(
            Layer::World,
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );
        let c = Model::new(
            Layer::World,
            Vector3::new(-1.0, 0.0, 12.35),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        );

        let a_idx = a.depth_index();
        let b_idx = b.depth_index();
        let c_idx = c.depth_index();

        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn transform_3d() {
        let a = Model::identity(Layer::World);
        let b = Model::identity(Layer::World);
        let c = Camera::default();

        assert_eq!(a.transform(&c, &b), Some(a.clone()));
        assert_eq!(b.transform(&c, &a), Some(a));
    }

    #[test]
    fn transform_2d() {
        let a = Model::identity(Layer::Ndc);
        let b = Model::identity(Layer::Ndc);
        let c = Camera::default();

        assert_eq!(a.transform(&c, &b), Some(a.clone()));
        assert_eq!(b.transform(&c, &a), Some(a));
    }

    #[test]
    fn transform_mixed() {
        let a = Model::identity(Layer::World);
        let b = Model::identity(Layer::Ndc);
        let c = Camera::default();

        assert_none!(b.transform(&c, &a));
        assert_some!(a.transform(&c, &b));
    }

    #[test]
    fn borrow() {
        let m = Model::default();
        let _: &Matrix4<f32> = m.borrow();
    }
}
