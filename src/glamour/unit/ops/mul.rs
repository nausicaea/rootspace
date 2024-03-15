use std::ops::Mul;

use crate::glamour::unit::Unit;

impl<'a, 'b, T> Mul<&'b Unit<T>> for &'a Unit<T>
where
    T: Into<Unit<T>>,
    &'a T: Mul<&'b T, Output = T>,
{
    type Output = Unit<T>;

    fn mul(self, rhs: &'b Unit<T>) -> Self::Output {
        Mul::mul(&self.0, &rhs.0).into()
    }
}

impl<T> Mul<Unit<T>> for Unit<T>
where
    T: Into<Unit<T>>,
    for<'a, 'b> &'a Unit<T>: Mul<&'b Unit<T>, Output = Unit<T>>,
{
    type Output = Unit<T>;

    fn mul(self, rhs: Unit<T>) -> Self::Output {
        Mul::mul(&self, &rhs)
    }
}

impl<'a, T> Mul<Unit<T>> for &'a Unit<T>
where
    T: Into<Unit<T>>,
    for<'b> &'a Unit<T>: Mul<&'b Unit<T>, Output = Unit<T>>,
{
    type Output = Unit<T>;

    fn mul(self, rhs: Unit<T>) -> Self::Output {
        Mul::mul(self, &rhs)
    }
}

impl<'b, T> Mul<&'b Unit<T>> for Unit<T>
where
    T: Into<Unit<T>>,
    for<'a> &'a Unit<T>: Mul<&'b Unit<T>, Output = Unit<T>>,
{
    type Output = Unit<T>;

    fn mul(self, rhs: &'b Unit<T>) -> Self::Output {
        Mul::mul(&self, rhs)
    }
}
