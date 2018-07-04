use super::{AsMatrix, DepthOrderingTrait};
use nalgebra::Matrix4;
use std::{f32, ops::Mul};

#[derive(Debug, Clone)]
pub struct Model(Matrix4<f32>);

impl Model {
    pub fn identity() -> Self {
        Model(Matrix4::identity())
    }
}

impl Default for Model {
    fn default() -> Self {
        Model::identity()
    }
}

impl AsRef<[[f32; 4]; 4]> for Model {
    fn as_ref(&self) -> &[[f32; 4]; 4] {
        self.0.as_ref()
    }
}

impl DepthOrderingTrait for Model {
    fn depth_index(&self) -> i32 {
        (self.0[(2, 3)] / f32::EPSILON).round() as i32
    }
}

impl AsMatrix for Model {
    fn as_matrix(&self) -> &Matrix4<f32> {
        &self.0
    }
}

impl<'a, 'b> Mul<&'b Model> for &'a Model {
    type Output = Model;

    fn mul(self, rhs: &'b Model) -> Self::Output {
        Model(self.0 * rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_matrix() {
        let m = Model::default();
        let _: &Matrix4<f32> = m.as_matrix();
    }
}
