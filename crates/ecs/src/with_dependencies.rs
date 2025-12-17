pub trait WithDependencies<D>: Sized {
    fn with_deps(deps: &D) -> anyhow::Result<Self>;
}
