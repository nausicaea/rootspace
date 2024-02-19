use crate::glamour::ops::cross::Cross;
use super::super::Unit;

impl<'a, 'b, T> Cross<&'b Unit<T>> for &'a Unit<T>
where
    &'a T: Cross<&'b T, Output = T>,
    T: Into<Unit<T>>,
{
    type Output = Unit<T>;

    fn cross(self, rhs: &'b Unit<T>) -> Self::Output {
        Cross::cross(&self.0, &rhs.0).into()
    }
}

impl<T> Cross<Self> for Unit<T>
where
    T: Cross<T, Output = T> + Into<Unit<T>>,
{
    type Output = Self;

    fn cross(self, rhs: Self) -> Self::Output {
        Cross::cross(self.0, rhs.0).into()
    }
}
