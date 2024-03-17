use serde::{Deserialize, Serialize};

use crate::{
    ecs::{component::Component, storage::vec_storage::VecStorage},
    glamour::{affine::Affine, mat::Mat4, num::ToMatrix, vec::Vec4},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTransform(Affine<f32>);

impl UiTransform {
    pub fn translation(&self) -> Vec4<f32> {
        Vec4::new(self.0.t.x, self.0.t.y, 0.0, 0.0)
    }

    pub fn scale(&self) -> f32 {
        self.0.s
    }

    pub fn depth(&self) -> f32 {
        self.0.t.z
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
            "translation: {}, depth: {}, scale: {}",
            self.translation(),
            self.depth(),
            self.scale(),
        )
    }
}
