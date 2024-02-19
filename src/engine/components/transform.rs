use serde::{Deserialize, Serialize};
use crate::ecs::component::Component;
use crate::ecs::storage::vec_storage::VecStorage;
use crate::glamour::affine::{Affine, AffineBuilder};
use crate::glamour::mat::Mat4;
use crate::glamour::quat::Quat;
use crate::glamour::unit::Unit;
use crate::glamour::vec::Vec4;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Transform(Affine<f32>);

impl Transform {
    pub fn builder() -> TransformBuilder {
        TransformBuilder::default()
    }

    pub fn look_at_lh<V: Into<Vec4<f32>>>(eye: V, cntr: V, up: V) -> Self {
        Transform(Affine::look_at_lh(eye.into(), cntr.into(), Unit::from(up.into())))
    }

    pub fn set_translation(&mut self, value: Vec4<f32>) {
        self.0.t = value;
    }

    pub fn set_orientation<Q: Into<Unit<Quat<f32>>>>(&mut self, value: Q) {
        self.0.o = value.into();
    }

    pub fn set_scale(&mut self, value: Vec4<f32>) {
        self.0.s = value;
    }

    pub fn as_affine(&self) -> &Affine<f32> {
        self.as_ref()
    }

    pub fn to_matrix(&self) -> Mat4<f32> {
        self.0.to_matrix()
    }

    pub fn translation(&self) -> &Vec4<f32> {
        &self.0.t
    }

    pub fn orientation(&self) -> &Unit<Quat<f32>> {
        &self.0.o
    }

    pub fn scale(&self) -> &Vec4<f32> {
        &self.0.s
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::builder().build()
    }
}

impl Component for Transform {
    type Storage = VecStorage<Self>;
}

impl From<Affine<f32>> for Transform {
    fn from(value: Affine<f32>) -> Self {
        Transform(value)
    }
}

impl AsRef<Affine<f32>> for Transform {
    fn as_ref(&self) -> &Affine<f32> {
        &self.0
    }
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "position: {}, orientation: {}, scale: {}",
            self.translation(),
            self.orientation(),
            self.scale(),
        )
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransformBuilder(AffineBuilder<f32>);

impl TransformBuilder {
    pub fn with_translation<V: Into<Vec4<f32>>>(mut self, t: V) -> Self {
        self.0 = self.0.with_translation(t.into());
        self
    }

    pub fn with_orientation<Q: Into<Quat<f32>>>(mut self, o: Q) -> Self {
        self.0 = self.0.with_orientation(o.into());
        self
    }

    pub fn with_scale<V: Into<Vec4<f32>>>(mut self, s: V) -> Self {
        self.0 = self.0.with_scale(s.into());
        self
    }

    pub fn build(self) -> Transform {
        Transform(self.0.build())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use proptest::{collection::vec, num::f32::NORMAL, prelude::*};
    use crate::glamour::num::{One, Zero};
    use crate::glamour::quat::Quat;
    use crate::glamour::vec::Vec4;

    use super::*;

    #[test]
    fn implements_default() {
        let _: Transform = Default::default();
    }

    #[test]
    fn provides_builder() {
        let _: TransformBuilder = Transform::builder();
    }

    #[test]
    fn blank_builder_is_the_same_as_default() {
        let ma: Transform = Transform::builder().build();
        let mb: Transform = Default::default();

        assert_eq!(ma, mb);
    }

    #[test]
    fn default_is_identity() {
        let m: Transform = Default::default();
        assert_ulps_eq!(m.to_matrix(), &Mat4::identity())
    }

    #[test]
    fn builder_accepts_position() {
        let _: TransformBuilder = TransformBuilder::default().with_translation(Vec4::zero());
    }

    #[test]
    fn builder_accepts_orientaton() {
        let _: TransformBuilder = TransformBuilder::default().with_orientation(Quat::identity());
    }

    #[test]
    fn builder_accepts_scale() {
        let _: TransformBuilder = TransformBuilder::default().with_scale(Vec4::zero());
    }

    #[test]
    fn builder_complete_example() {
        let m: Transform = TransformBuilder::default()
            .with_translation(Vec4::zero())
            .with_orientation(Quat::identity())
            .with_scale(Vec4::one())
            .build();

        assert_ulps_eq!(m.translation(), &Vec4::zero());
        assert_ulps_eq!(m.orientation(), &Unit::from(Quat::identity()));
        assert_ulps_eq!(m.scale(), &Vec4::one());
    }

    proptest! {
        #[test]
        fn position_may_be_changed(num in vec(NORMAL, 3)) {
            let mut m = Transform::default();

            let p = Vec4::new(num[0], num[1], num[2], 0.0);
            m.set_translation(p.clone());

            prop_assert_eq!(m.translation(), &p);
        }

        #[test]
        fn orientation_may_be_changed(num in vec(NORMAL, 4)) {
            let mut m = Transform::default();

            let o = Unit::from(Quat::new(num[0], num[1], num[2], num[3]));
            m.set_orientation(o.clone());

            prop_assert_eq!(m.orientation(), &o);
        }

        #[test]
        fn scale_may_be_changed(num in vec(NORMAL, 3)) {
            let mut m = Transform::default();

            let s = Vec4::new(num[0], num[1], num[2], 0.0);
            m.set_scale(s.clone());

            prop_assert_eq!(m.scale(), &s);
        }
    }
}
