use std::f32;
use std::ops::Mul;

pub trait DepthOrderingTrait {
    fn depth_index(&self) -> i32;
}

#[derive(Clone, Default)]
pub struct Model(f32);

impl DepthOrderingTrait for Model {
    fn depth_index(&self) -> i32 {
        (self.0 / f32::EPSILON).round() as i32
    }
}

impl<'a, 'b> Mul<&'b Model> for &'a Model {
    type Output = Model;

    fn mul(self, rhs: &'b Model) -> Self::Output {
        Model(self.0 * rhs.0)
    }
}
