use std::{iter::Product, ops::Mul};

use ecs::{Component, VecStorage};
use forward_ref::forward_ref_binop;
use glamour::{Affine, AffineBuilder, Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiModel(Affine<f32>);

impl UiModel {
    pub fn builder() -> UiModelBuilder {
        UiModelBuilder::default()
    }

    pub fn set_translation(&mut self, value: Vec2<f32>) {
        self.0.t[0] = value.x();
        self.0.t[1] = value.y();
    }

    pub fn set_scale(&mut self, value: Vec2<f32>) {
        self.0.s = Vec3::new(value.x(), value.y(), 1.0);
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

    pub fn translation(&self) -> Vec2<f32> {
        Vec2::new(self.0.t.x(), self.0.t.y())
    }

    pub fn scale(&self) -> Vec2<f32> {
        Vec2::new(self.0.s.x(), self.0.s.y())
    }

    pub fn depth(&self) -> f32 {
        self.0.t.z()
    }
}

impl Default for UiModel {
    fn default() -> Self {
        UiModel::builder().build()
    }
}

impl Component for UiModel {
    type Storage = VecStorage<Self>;
}

impl From<Affine<f32>> for UiModel {
    fn from(value: Affine<f32>) -> Self {
        UiModel(value)
    }
}

impl AsRef<Affine<f32>> for UiModel {
    fn as_ref(&self) -> &Affine<f32> {
        &self.0
    }
}

impl std::fmt::Display for UiModel {
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

impl<'a, 'b> Mul<&'b UiModel> for &'a UiModel {
    type Output = UiModel;

    fn mul(self, rhs: &'b UiModel) -> UiModel {
        UiModel(self.as_affine().mul(rhs.as_affine()))
    }
}

forward_ref_binop!(impl Mul, mul for UiModel, UiModel, UiModel);

impl Product for UiModel {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(UiModel::default(), |state, item| state * item)
    }
}

impl<'a> Product<&'a UiModel> for UiModel {
    fn product<I: Iterator<Item = &'a UiModel>>(iter: I) -> Self {
        iter.fold(UiModel::default(), |state, item| state * item)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UiModelBuilder {
    t: Option<Vec2<f32>>,
    d: Option<f32>,
    s: Option<Vec2<f32>>,
}

impl UiModelBuilder {
    pub fn with_translation(mut self, t: Vec2<f32>) -> Self {
        self.t = Some(t);
        self
    }

    pub fn with_depth(mut self, d: f32) -> Self {
        self.d = Some(d);
        self
    }

    pub fn with_scale(mut self, s: Vec2<f32>) -> Self {
        self.s = Some(s);
        self
    }

    pub fn build(self) -> UiModel {
        let t: Vec3<f32> = self
            .t
            .zip(self.d)
            .map(|(t, d)| Vec3::new(t.x(), t.y(), d))
            .unwrap_or_else(Vec3::zero);
        let s: Vec3<f32> = self.s.map(|s| Vec3::new(s.x(), s.y(), 1.0)).unwrap_or_else(Vec3::one);
        UiModel(AffineBuilder::default().with_translation(t).with_scale(s).build())
    }
}
