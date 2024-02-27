pub trait WithDependencies<D>: Sized {
    async fn with_deps(deps: &D) -> Result<Self, anyhow::Error>;
}
