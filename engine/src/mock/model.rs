use std::f32;
use std::ops::Mul;
use math::DepthOrderingTrait;

#[derive(Clone, Default)]
pub struct MockModel(f32);

impl MockModel {
    pub fn new(z: f32) -> MockModel {
        MockModel(z)
    }
}

impl DepthOrderingTrait for MockModel {
    fn depth_index(&self) -> i32 {
        (self.0 / f32::EPSILON).round() as i32
    }
}

impl<'a, 'b> Mul<&'b MockModel> for &'a MockModel {
    type Output = MockModel;

    fn mul(self, rhs: &'b MockModel) -> Self::Output {
        MockModel(self.0 * rhs.0)
    }
}

