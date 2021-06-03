use crate::short_type_name;

// TODO: Solve this differently; perhaps by accessing a property in Serialize/Deserialize?
pub trait SerializationName: Sized {
    fn name() -> String {
        short_type_name::<Self>()
    }
}

impl SerializationName for () {}
