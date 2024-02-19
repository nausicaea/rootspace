use anyhow::Error;

use super::resources::Resources;

pub trait WithResources: Sized {
    fn with_res(res: &Resources) -> Result<Self, Error>;
}
