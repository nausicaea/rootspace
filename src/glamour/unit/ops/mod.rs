use super::Unit;

pub mod cross;
pub mod dot;
pub mod mul;

impl<T> AsRef<T> for Unit<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::Deref for Unit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
