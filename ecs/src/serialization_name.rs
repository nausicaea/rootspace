use crate::short_type_name;
use std::fmt::{Debug, Display};

// TODO: Solve this differently; perhaps by accessing a property in Serialize/Deserialize?
pub trait SerializationName: Sized {
    fn name() -> String {
        short_type_name::<Self>()
    }
}

impl SerializationName for () {}
