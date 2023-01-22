use std::{iter::Product, ops::Mul};

use ecs::{Component, VecStorage};
use forward_ref::forward_ref_binop;
use glamour::{Affine, AffineBuilder, Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTransform(Affine<f32>);

impl UiTransform {
    pub fn builder() -> UiTransformBuilder {
        UiTransformBuilder::default()
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

impl<'a, 'b> Mul<&'b UiTransform> for &'a UiTransform {
    type Output = UiTransform;

    fn mul(self, rhs: &'b UiTransform) -> UiTransform {
        UiTransform(self.as_affine().mul(rhs.as_affine()))
    }
}

forward_ref_binop!(impl Mul, mul for UiTransform, UiTransform, UiTransform);

impl Product for UiTransform {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(UiTransform::default(), |state, item| state * item)
    }
}

impl<'a> Product<&'a UiTransform> for UiTransform {
    fn product<I: Iterator<Item = &'a UiTransform>>(iter: I) -> Self {
        iter.fold(UiTransform::default(), |state, item| state * item)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UiTransformBuilder {
    t: Option<Vec2<f32>>,
    d: Option<f32>,
    s: Option<Vec2<f32>>,
}

impl UiTransformBuilder {
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

    pub fn build(self) -> UiTransform {
        let t: Vec3<f32> = self
            .t
            .zip(self.d)
            .map(|(t, d)| Vec3::new(t.x(), t.y(), d))
            .unwrap_or_else(Vec3::zero);
        let s: Vec3<f32> = self.s.map(|s| Vec3::new(s.x(), s.y(), 1.0)).unwrap_or_else(Vec3::one);
        UiTransform(AffineBuilder::default().with_translation(t).with_scale(s).build())
    }
}
