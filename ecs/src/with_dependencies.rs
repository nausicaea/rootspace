use anyhow::Error;

pub trait WithDependencies<D>: Sized {
    fn with_deps(deps: &D) -> Result<Self, Error>;
}
