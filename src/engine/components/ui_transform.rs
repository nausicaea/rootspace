use crate::ecs::component::Component;
use crate::ecs::storage::vec_storage::VecStorage;
use crate::glamour::affine::{Affine, AffineBuilder};
use crate::glamour::mat::Mat4;
use crate::glamour::num::Zero;
use crate::glamour::vec::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTransform(Affine<f32>);

impl UiTransform {
    pub fn builder() -> UiTransformBuilder {
        UiTransformBuilder::default()
    }

    pub fn set_translation(&mut self, value: Vec4<f32>) {
        self.0.t[0] = value.x;
        self.0.t[1] = value.y;
    }

    pub fn set_scale(&mut self, value: Vec4<f32>) {
        self.0.s = Vec4::new(value.x, value.y, 1.0, 0.0);
    }

    pub fn set_depth(&mut self, value: f32) {
        self.0.t[2] = value;
    }

    pub fn as_affine(&self) -> &Affine<f32> {
        self.as_ref()
    }

    pub fn to_matrix(&self) -> Mat4<f32> {
        self.0.to_matrix()
    }

    pub fn translation(&self) -> Vec4<f32> {
        Vec4::new(self.0.t.x, self.0.t.y, 0.0, 0.0)
    }

    pub fn scale(&self) -> Vec4<f32> {
        Vec4::new(self.0.s.x, self.0.s.y, 0.0, 0.0)
    }

    pub fn depth(&self) -> f32 {
        self.0.t.z
    }
}

impl Default for UiTransform {
    fn default() -> Self {
        UiTransform::builder().build()
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

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UiTransformBuilder {
    t: Option<Vec4<f32>>,
    d: Option<f32>,
    s: Option<Vec4<f32>>,
}

impl UiTransformBuilder {
    pub fn with_translation(mut self, t: Vec4<f32>) -> Self {
        self.t = Some(t);
        self
    }

    pub fn with_depth(mut self, d: f32) -> Self {
        self.d = Some(d);
        self
    }

    pub fn with_scale(mut self, s: Vec4<f32>) -> Self {
        self.s = Some(s);
        self
    }

    pub fn build(self) -> UiTransform {
        let t: Vec4<f32> = self
            .t
            .zip(self.d)
            .map(|(t, d)| Vec4::new(t.x, t.y, d, 0.0))
            .unwrap_or_else(Vec4::zero);
        let s: Vec4<f32> = self
            .s
            .map(|s| Vec4::new(s.x, s.y, 1.0, 0.0))
            .unwrap_or_else(|| Vec4::new(1.0, 1.0, 1.0, 0.0));
        UiTransform(AffineBuilder::default().with_translation(t).with_scale(s).build())
    }
}
