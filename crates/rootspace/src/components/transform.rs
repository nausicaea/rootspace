use serde::{Deserialize, Serialize};

use ecs::{component::Component, storage::vec_storage::VecStorage};
use glamour::{
    affine::{builder::AffineBuilder, Affine},
    mat::Mat4,
    num::ToMatrix,
    quat::Quat,
    unit::Unit,
    vec::Vec4,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub(crate) affine: Affine<f32>,
    pub(crate) ui: bool,
}

impl Transform {
    pub fn builder() -> TransformBuilder {
        TransformBuilder::default()
    }

    pub fn look_at_rh<V1, V2, V3>(eye: V1, cntr: V2, up: V3) -> Self
    where
        V1: Into<Vec4<f32>>,
        V2: Into<Vec4<f32>>,
        V3: Into<Vec4<f32>>,
    {
        Transform {
            affine: Affine::with_look_at_rh(eye.into(), cntr.into(), Unit::from(up.into())),
            ui: false,
        }
    }

    pub fn look_at_rh_inv<V1, V2, V3>(eye: V1, cntr: V2, up: V3) -> Self
    where
        V1: Into<Vec4<f32>>,
        V2: Into<Vec4<f32>>,
        V3: Into<Vec4<f32>>,
    {
        use num_traits::Inv;
        Transform {
            affine: Affine::with_look_at_rh(eye.into(), cntr.into(), Unit::from(up.into())).inv(),
            ui: false,
        }
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
        Transform {
            affine: value,
            ui: false,
        }
    }
}

impl AsRef<Affine<f32>> for Transform {
    fn as_ref(&self) -> &Affine<f32> {
        &self.affine
    }
}

impl ToMatrix<f32> for Transform {
    fn to_matrix(&self) -> Mat4<f32> {
        self.affine.to_matrix()
    }
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "position: {}, orientation: {}, scale: {}",
            self.affine.t, self.affine.o, self.affine.s,
        )
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TransformBuilder {
    affine_builder: AffineBuilder<f32>,
    ui: bool,
}

impl TransformBuilder {
    pub fn with_translation<V: Into<Vec4<f32>>>(mut self, t: V) -> Self {
        self.affine_builder = self.affine_builder.with_translation(t.into());
        self
    }

    pub fn with_orientation<Q: Into<Unit<Quat<f32>>>>(mut self, o: Q) -> Self {
        self.affine_builder = self.affine_builder.with_orientation(o.into());
        self
    }

    pub fn with_scale(mut self, s: f32) -> Self {
        self.affine_builder = self.affine_builder.with_scale(s);
        self
    }

    pub fn with_ui(mut self, ui: bool) -> Self {
        self.ui = ui;
        self
    }

    pub fn build(self) -> Transform {
        Transform {
            affine: self.affine_builder.build(),
            ui: self.ui,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glamour::{num::Zero, quat::Quat, vec::Vec4};

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
        assert_eq!(m.to_matrix(), Mat4::identity())
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
        let _: TransformBuilder = TransformBuilder::default().with_scale(0.0f32);
    }

    #[test]
    fn builder_complete_example() {
        let m: Transform = TransformBuilder::default()
            .with_translation(Vec4::zero())
            .with_orientation(Quat::identity())
            .with_scale(1.0f32)
            .build();

        assert_eq!(m.affine.t, Vec4::zero());
        assert_eq!(m.affine.o, Unit::from(Quat::identity()));
        assert_eq!(m.affine.s, 1.0f32);
    }
}
