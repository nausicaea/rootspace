pub trait WithDependencies<D>: Sized {
    fn with_deps(deps: &D) -> impl Future<Output = anyhow::Result<Self>>;
}
