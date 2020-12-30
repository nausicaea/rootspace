use ecs::{Component, VecStorage};
use nalgebra::{zero, Affine3, Isometry3, Matrix4, Point2, Vector2, Vector3};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UiModelSerDe {
    translation: Vector2<f32>,
    scale: Vector2<f32>,
    depth: f32,
}

impl From<UiModel> for UiModelSerDe {
    fn from(value: UiModel) -> Self {
        UiModelSerDe {
            translation: value.translation,
            scale: value.scale,
            depth: value.depth,
        }
    }
}

impl From<UiModelSerDe> for UiModel {
    fn from(value: UiModelSerDe) -> Self {
        UiModel::new(value.translation, value.scale, value.depth)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "UiModelSerDe", from = "UiModelSerDe")]
pub struct UiModel {
    model: Affine3<f32>,
    translation: Vector2<f32>,
    scale: Vector2<f32>,
    depth: f32,
}

impl UiModel {
    pub fn new(translation: Vector2<f32>, scale: Vector2<f32>, depth: f32) -> Self {
        let isometry = Isometry3::new(Vector3::new(translation.x, translation.y, depth), zero());
        let scale_matrix = Affine3::from_matrix_unchecked(Matrix4::new(
            scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ));

        UiModel {
            model: isometry * scale_matrix,
            translation,
            scale,
            depth,
        }
    }

    pub fn identity() -> Self {
        UiModel {
            model: Affine3::identity(),
            translation: Vector2::new(0.0, 0.0),
            scale: Vector2::new(1.0, 1.0),
            depth: 0.0,
        }
    }

    pub fn set_position(&mut self, value: Point2<f32>) {
        self.translation = value.coords;
        self.recalculate_matrix();
    }

    pub fn set_scale(&mut self, value: Vector2<f32>) {
        self.scale = value;
        self.recalculate_matrix();
    }

    pub fn set_depth(&mut self, value: f32) {
        self.depth = value;
        self.recalculate_matrix();
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        self.model.matrix()
    }

    pub fn position(&self) -> Point2<f32> {
        Point2::from(self.translation)
    }

    pub fn scale(&self) -> &Vector2<f32> {
        &self.scale
    }

    pub fn depth(&self) -> f32 {
        self.depth
    }

    fn recalculate_matrix(&mut self) {
        let isometry = Isometry3::new(
            Vector3::new(self.translation.x, self.translation.y, self.depth),
            zero(),
        );
        let scale_matrix = Affine3::from_matrix_unchecked(Matrix4::new(
            self.scale.x,
            0.0,
            0.0,
            0.0,
            0.0,
            self.scale.y,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ));

        self.model = isometry * scale_matrix;
    }
}

impl Default for UiModel {
    fn default() -> Self {
        UiModel::identity()
    }
}

impl Component for UiModel {
    type Storage = VecStorage<Self>;
}

impl Mul<UiModel> for UiModel {
    type Output = UiModel;

    fn mul(self, rhs: UiModel) -> UiModel {
        &self * &rhs
    }
}

impl<'a, 'b> Mul<&'a UiModel> for &'b UiModel {
    type Output = UiModel;

    fn mul(self, rhs: &'a UiModel) -> UiModel {
        UiModel::new(
            self.translation + rhs.translation,
            self.scale.component_mul(&rhs.scale),
            self.depth + rhs.depth,
        )
    }
}
