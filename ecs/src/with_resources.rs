use crate::resources::Resources;

pub trait FromResources: Sized {
    fn from_resources(res: &Resources) -> Self;
}