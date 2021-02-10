use crate::resources::Resources;

pub trait WithResources: Sized {
    fn with_resources(res: &Resources) -> Self;
}

impl WithResources for () {
    fn with_resources(_: &Resources) -> () {
        ()
    }
}