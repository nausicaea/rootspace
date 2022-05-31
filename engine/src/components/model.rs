use ecs::{Component, VecStorage};
use glamour::{Affine, Mat4, Vec3, Quat, AffineBuilder, Unit};
use forward_ref::forward_ref_binop;
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use std::iter::Product;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Model(Affine<f32>);

impl Model {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::default()
    }

    pub fn set_translation(&mut self, value: Vec3<f32>) {
        self.0.t = value;
    }

    pub fn set_orientation<Q: Into<Unit<Quat<f32>>>>(&mut self, value: Q) {
        self.0.o = value.into();
    }

    pub fn set_scale(&mut self, value: Vec3<f32>) {
        self.0.s = value;
    }

    pub fn as_affine(&self) -> &Affine<f32> {
        self.as_ref()
    }

    pub fn to_matrix(&self) -> Mat4<f32> {
        self.0.to_matrix()
    }

    pub fn translation(&self) -> &Vec3<f32> {
        &self.0.t
    }

    pub fn orientation(&self) -> &Unit<Quat<f32>> {
        &self.0.o
    }

    pub fn scale(&self) -> &Vec3<f32> {
        &self.0.s
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

impl From<Affine<f32>> for Model {
    fn from(value: Affine<f32>) -> Self {
        Model(value)
    }
}

impl AsRef<Affine<f32>> for Model {
    fn as_ref(&self) -> &Affine<f32> {
        &self.0
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "position: [{}, {}, {}], orientation: {}, scale: [{}, {}, {}]",
            self.translation().x(),
            self.translation().y(),
            self.translation().z(),
            self.orientation(),
            self.scale().x(),
            self.scale().y(),
            self.scale().z(),
        )
    }
}

impl<'a, 'b> Mul<&'b Model> for &'a Model {
    type Output = Model;

    fn mul(self, rhs: &'b Model) -> Self::Output {
        Model(self.as_affine().mul(rhs.as_affine()))
    }
}

forward_ref_binop!(impl Mul, mul for Model, Model, Model);

impl Product for Model {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Model::default(), |state, item| state * item)
    }
}

impl<'a> Product<&'a Model> for Model {
    fn product<I: Iterator<Item = &'a Model>>(iter: I) -> Self {
        iter.fold(Model::default(), |state, item| state * item)
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ModelBuilder(AffineBuilder<f32>);

impl ModelBuilder {
    pub fn with_translation(mut self, t: Vec3<f32>) -> Self {
        self.0 = self.0.with_translation(t);
        self
    }

    pub fn with_orientation(mut self, o: Quat<f32>) -> Self {
        self.0 = self.0.with_orientation(o);
        self
    }

    pub fn with_scale(mut self, s: Vec3<f32>) -> Self {
        self.0 = self.0.with_scale(s);
        self
    }

    pub fn build(self) -> Model {
        Model(self.0.build())
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_ulps_eq};
    use proptest::prelude::*;
    use proptest::collection::vec;
    use proptest::num::f32::NORMAL;

    use super::*;

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
    }

    #[test]
    fn default_is_identity() {
        let m: Model = Default::default();
        assert_ulps_eq!(m.to_matrix(), &Mat4::identity())
    }

    #[test]
    fn builder_accepts_position() {
        let _: ModelBuilder = ModelBuilder::default().with_translation(Vec3::zero());
    }

    #[test]
    fn builder_accepts_orientaton() {
        let _: ModelBuilder =
            ModelBuilder::default().with_orientation(Quat::identity());
    }

    #[test]
    fn builder_accepts_scale() {
        let _: ModelBuilder = ModelBuilder::default().with_scale(Vec3::zero());
    }

    #[test]
    fn builder_complete_example() {
        let m: Model = ModelBuilder::default()
            .with_translation(Vec3::zero())
            .with_orientation(Quat::identity())
            .with_scale(Vec3::one())
            .build();

        assert_ulps_eq!(m.translation(), &Vec3::zero());
        assert_ulps_eq!(m.orientation(), &Unit::from(Quat::identity()));
        assert_ulps_eq!(m.scale(), &Vec3::one());
    }

    proptest! {
        #[test]
        fn position_may_be_changed(num in vec(NORMAL, 3)) {
            let mut m = Model::default();

            let p = Vec3::new(num[0], num[1], num[2]);
            m.set_translation(p.clone());

            prop_assert_eq!(m.translation(), &p);
        }

        #[test]
        fn orientation_may_be_changed(num in vec(NORMAL, 4)) {
            let mut m = Model::default();

            let o = Unit::from(Quat::new(num[0], num[1], num[2], num[3]));
            m.set_orientation(o.clone());

            prop_assert_eq!(m.orientation(), &o);
        }

        #[test]
        fn scale_may_be_changed(num in vec(NORMAL, 3)) {
            let mut m = Model::default();

            let s = Vec3::new(num[0], num[1], num[2]);
            m.set_scale(s.clone());

            prop_assert_eq!(m.scale(), &s);
        }
    }
}
