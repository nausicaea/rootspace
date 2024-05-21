pub trait WithDependencies<D>: Sized {
    fn with_deps(deps: &D) -> impl std::future::Future<Output = Result<Self, anyhow::Error>>;
}
