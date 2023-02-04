use super::super::Unit;
use crate::Dot;

impl<'a, 'b, T> Dot<&'b Unit<T>> for &'a Unit<T>
where
    T: Into<Unit<T>>,
    &'a T: Dot<&'b T, Output = T>,
{
    type Output = Unit<T>;

    fn dot(self, rhs: &'b Unit<T>) -> Self::Output {
        Dot::dot(&self.0, &rhs.0).into()
    }
}
