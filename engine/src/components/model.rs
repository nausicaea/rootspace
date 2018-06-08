use std::ops::Mul;

#[derive(Clone, Default)]
pub struct Model(f32);

impl<'a, 'b> Mul<&'b Model> for &'a Model {
    type Output = Model;

    fn mul(self, rhs: &'b Model) -> Self::Output {
        Model(self.0 * rhs.0)
    }
}
