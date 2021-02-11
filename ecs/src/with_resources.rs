use crate::resources::Resources;

pub trait WithResources: Sized {
    fn with_resources(res: &Resources) -> Self;
}

impl<T> WithResources for T
where
    T: Default,
{
    fn with_resources(_: &Resources) -> T {
        T::default()
    }
}