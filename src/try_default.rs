use anyhow::Error;

/// Implementors provide a parameterless constructor that may return a value or an error
pub trait TryDefault: Sized {
    fn try_default() -> Result<Self, Error>;
}

impl<T> TryDefault for T
where
    T: Default,
{
    fn try_default() -> Result<Self, Error> {
        Ok(Default::default())
    }
}
