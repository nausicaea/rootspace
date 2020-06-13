/// Implementors provide a parameterless constructor that may return a value
pub trait MaybeDefault: Sized {
    fn maybe_default() -> Option<Self> {
        None
    }
}

impl<T> MaybeDefault for T
where
    T: Default
{
    fn maybe_default() -> Option<Self> {
        Some(Default::default())
    }
}
