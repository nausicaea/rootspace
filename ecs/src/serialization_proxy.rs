use crate::short_type_name;
use std::fmt::{Debug, Display};

#[derive(Copy, Clone, Debug)]
pub struct EmptyProxyError;

impl Display for EmptyProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(std::any::type_name::<Self>())
    }
}

pub trait SerializationProxy: Sized {
    fn name() -> String {
        short_type_name::<Self>()
    }
}

impl SerializationProxy for () {}
