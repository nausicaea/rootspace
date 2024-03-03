use anyhow::Error;

use super::resources::Resources;

pub trait WithResources: Sized + Send {
    fn with_res(res: &Resources) -> impl std::future::Future<Output = Result<Self, Error>> + Send;
}

impl WithResources for () {
    async fn with_res(_: &Resources) -> Result<Self, Error> {
        Ok(())
    }
}
