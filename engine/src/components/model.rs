use super::DepthOrderingTrait;
use affine_transform::AffineTransform;
use nalgebra::{Matrix4, Vector3, Affine3, Isometry3};
use std::{f32, ops::Mul};
use std::borrow::Borrow;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    model: Affine3<f32>,
    decomposed: AffineTransform<f32>,
}

impl Model {
    pub fn new(translation: Vector3<f32>, axisangle: Vector3<f32>, scale: Vector3<f32>) -> Self {
        let isometry = Isometry3::new(translation, axisangle);
        let scale_matrix = Affine3::from_matrix_unchecked(Matrix4::new(scale.x, 0.0, 0.0, 0.0,
                                                                       0.0, scale.y, 0.0, 0.0,
                                                                       0.0, 0.0, scale.z, 0.0,
                                                                       0.0, 0.0, 0.0, 1.0));

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
}

impl Default for Model {
    fn default() -> Self {
        Model::identity()
    }
}

impl DepthOrderingTrait for Model {
    fn depth_index(&self) -> i32 {
        (self.decomposed.translation.vector.z / f32::EPSILON).round() as i32
    }
}

impl Borrow<Matrix4<f32>> for Model {
    fn borrow(&self) -> &Matrix4<f32> {
        self.model.matrix()
    }
}

impl<'a, 'b> Mul<&'b Model> for &'a Model {
    type Output = Model;

    fn mul(self, rhs: &'b Model) -> Self::Output {
        let product = self.model * rhs.model;

        Model {
            model: product,
            decomposed: product.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _ = Model::new(Vector3::y(), Vector3::z(), Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn identity() {
        let ident = Model::identity();
        let ident_mat: &Matrix4<f32> = ident.borrow();
        assert_eq!(ident_mat, &Matrix4::identity());
    }

    #[test]
    fn default() {
        assert_eq!(Model::default(), Model::identity());
    }

    #[test]
    fn depth_ordering() {
        let a = Model::new(Vector3::new(-1.0, 0.0, -10.35), Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let b = Model::new(Vector3::new(-1.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let c = Model::new(Vector3::new(-1.0, 0.0, 12.35), Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));

        let a_idx = a.depth_index();
        let b_idx = b.depth_index();
        let c_idx = c.depth_index();

        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn multiplication() {
        let a = Model::new(Vector3::new(-1.0, 0.0, -10.35), Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let b = Model::identity();

        assert_eq!(&a * &b, a);
    }

    #[test]
    fn borrow() {
        let m = Model::default();
        let _: &Matrix4<f32> = m.borrow();
    }
}
