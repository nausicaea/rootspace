use super::resources::Resources;

pub trait WithResources: Sized + Send {
    fn with_res(res: &Resources) -> impl std::future::Future<Output = anyhow::Result<Self>> + Send;
}

impl WithResources for () {
    #[tracing::instrument(skip_all)]
    async fn with_res(_: &Resources) -> anyhow::Result<Self> {
        Ok(())
    }
}
