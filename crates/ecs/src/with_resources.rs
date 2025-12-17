use super::resources::Resources;

pub trait WithResources: Sized + Send {
    fn with_res(res: &Resources) -> anyhow::Result<Self>;
}

impl WithResources for () {
    #[tracing::instrument(skip_all)]
    fn with_res(_: &Resources) -> anyhow::Result<Self> {
        Ok(())
    }
}
