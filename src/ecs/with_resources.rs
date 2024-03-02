use anyhow::Error;

use super::resources::Resources;

pub trait WithResources: Sized {
    async fn with_res(res: &Resources) -> Result<Self, Error>;
}

impl WithResources for () {
    async fn with_res(_: &Resources) -> Result<Self, Error> {
        Ok(())
    }
}