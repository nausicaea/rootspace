use anyhow::Result;

/// Implementors provide a parameterless constructor that may return a value or an error
pub trait TryDefault: Sized {
    fn try_default() -> Result<Self>;
}

impl<T> TryDefault for T
    where
        T: Default,
{
    fn try_default() -> Result<Self> {
        Ok(Default::default())
    }
}
