use serde::{Deserialize, Serialize};

use crate::{
    ecs::{component::Component, storage::vec_storage::VecStorage},
    glamour::{affine::Affine, mat::Mat4, num::ToMatrix, unit::Unit, vec::Vec4},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTransform(pub(crate) Affine<f32>);

impl UiTransform {
    pub fn look_at_lh<V: Into<Vec4<f32>>>(eye: V, cntr: V, up: V) -> Self {
        UiTransform(Affine::with_look_at_lh(eye.into(), cntr.into(), Unit::from(up.into())))
    }
}

impl Default for UiTransform {
    fn default() -> Self {
        UiTransform(Affine::identity())
    }
}

impl Component for UiTransform {
    type Storage = VecStorage<Self>;
}

impl From<Affine<f32>> for UiTransform {
    fn from(value: Affine<f32>) -> Self {
        UiTransform(value)
    }
}

impl AsRef<Affine<f32>> for UiTransform {
    fn as_ref(&self) -> &Affine<f32> {
        &self.0
    }
}

impl ToMatrix<f32> for UiTransform {
    fn to_matrix(&self) -> Mat4<f32> {
        self.0.to_matrix()
    }
}

impl std::fmt::Display for UiTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "translation: ({}, {}), depth: {}, scale: {}",
            self.0.t.x, self.0.t.y, self.0.t.z, self.0.s,
        )
    }
}
