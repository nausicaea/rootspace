use std::ops::Mul;

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

    pub fn orientation(&self) -> UnitQuaternion<f32> {
        self.decomposed.rotation
    }

    pub fn scale(&self) -> Vector3<f32> {
        self.decomposed.scale
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
    use crate::utilities::validate_float;
    use approx::{assert_ulps_eq, ulps_eq};
    use nalgebra::{one, zero, Point4, Quaternion, Unit, Vector4};
    use proptest::prelude::*;

    #[test]
    fn implements_default() {
        let _: Model = Default::default();
    }

    #[test]
    fn provides_builder() {
        let _: ModelBuilder = Model::builder();
    }

    #[test]
    fn blank_builder_is_the_same_as_default() {
        let ma: Model = Model::builder().build();
        let mb: Model = Default::default();

        assert_eq!(ma, mb);
        // TODO: assert_ulps_eq!(ma, mb);
    }

    #[test]
    fn default_is_identity() {
        let m: Model = Default::default();
        assert_ulps_eq!(m.matrix(), &Matrix4::identity())
    }

    #[test]
    fn builder_accepts_position() {
        let _: ModelBuilder = ModelBuilder::default().with_position(Point3::from(Vector3::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn builder_accepts_orientaton() {
        let _: ModelBuilder =
            ModelBuilder::default().with_orientation(Unit::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)));
    }

    #[test]
    fn builder_accepts_scale() {
        let _: ModelBuilder = ModelBuilder::default().with_scale(Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn builder_complete_example() {
        let m: Model = ModelBuilder::default()
            .with_position(Point3::from([zero(); 3]))
            .with_orientation(Unit::new_normalize(Quaternion::identity()))
            .with_scale(Vector3::from([zero(); 3]))
            .build();

        assert_ulps_eq!(m.position(), Point3::from([zero(); 3]));
        assert_ulps_eq!(m.orientation(), Unit::new_normalize(Quaternion::identity()));
        assert_ulps_eq!(m.scale(), Vector3::from([zero(); 3]));
    }

    #[test]
    fn transform_point_works_for_zeroes() {
        let m: Model = ModelBuilder::default()
            .with_position(Point3::from([zero(); 3]))
            .with_orientation(Unit::new_normalize(Quaternion::identity()))
            .with_scale(Vector3::from([one(); 3]))
            .build();
        let p: Point3<f32> = Point3::from([zero(); 3]);

        let tpt: Point3<f32> = m.transform_point(&p);
        let tpt: Vector4<f32> = Vector4::new(tpt.x, tpt.y, tpt.z, one());
        let mmul = m.matrix() * Vector4::new(p.x, p.y, p.z, one());

        assert_ulps_eq!(tpt, mmul);
    }

    proptest! {
        #[test]
        fn position_may_be_changed(num: [f32; 3]) {
            let mut m = Model::default();

            let p = Point3::from_slice(&num);
            m.set_position(p);

            if !validate_float(&num) {
                return Ok(());
            } else {
                prop_assert_eq!(m.position(), p);
            }
        }

        #[test]
        fn orientation_may_be_changed(num: [f32; 4]) {
            let mut m = Model::default();

            let o = Unit::new_normalize(Quaternion::new(num[0], num[1], num[2], num[3]));
            m.set_orientation(o);

            if !validate_float(&num) {
                return Ok(());
            } else {
                prop_assert_eq!(m.orientation(), o);
            }
        }

        #[test]
        fn scale_may_be_changed(num: [f32; 3]) {
            let mut m = Model::default();

            let s = Vector3::new(num[0], num[1], num[2]);
            m.set_scale(s);

            if !validate_float(&num) {
                return Ok(());
            } else {
                prop_assert_eq!(m.scale(), s);
            }
        }

        #[test]
        fn transform_point_is_the_same_as_matrix_multiplication(num: [f32; 13]) {
            let m = Model::builder()
                .with_position(Point3::from_slice(&num[0..3]))
                .with_orientation(Unit::new_normalize(Quaternion::from_vector(Point4::from_slice(&num[3..7]).coords)))
                .with_scale(Point3::from_slice(&num[7..10]).coords)
                .build();
            let p = Point3::from_slice(&num[10..13]);

            if !validate_float(&num) {
                return Ok(())
            } else {
                let tpt: Point3<f32> = m.transform_point(&p);
                let tpt: Vector4<f32> = Vector4::new(tpt.x, tpt.y, tpt.z, one());
                let mmul = m.matrix() * Vector4::new(p.x, p.y, p.z, one());

                prop_assert!(ulps_eq!(tpt, mmul), "{:?} != {:?}", tpt, mmul);
            }
        }

        #[test]
        fn transformations_are_invertible(num: [f32; 13]) {
            let m = Model::builder()
                .with_position(Point3::from_slice(&num[0..3]))
                .with_orientation(Unit::new_normalize(Quaternion::from_vector(Point4::from_slice(&num[3..7]).coords)))
                .with_scale(Point3::from_slice(&num[7..10]).coords)
                .build();
            let p = Point3::from_slice(&num[10..13]);

            if !validate_float(&num) {
                return Ok(())
            } else {
                let tpt: Point3<f32> = m.transform_point(&p);
                let itpt: Point3<f32> = m.inverse_transform_point(&tpt);

                prop_assert!(ulps_eq!(p, itpt), "{:?} != {:?}", p, itpt);
            }
        }
    }
}
