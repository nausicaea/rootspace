use crate::registry::SystemRegistry;
use super::Systems;
use std::marker::PhantomData;
use serde::ser::{self, Serializer, SerializeSeq};
use serde::Serialize;
use log::debug;
use crate::short_type_name::short_type_name;

#[derive(Debug, Serialize)]
struct TaggedRef<'a, T> {
    t: &'static str,
    c: &'a T,
}

impl<'a, T> std::ops::Deref for TaggedRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.c
    }
}

impl<'a, T> From<&'a T> for TaggedRef<'a, T> {
    fn from(value: &'a T) -> Self {
        TaggedRef {
            t: std::any::type_name::<T>(),
            c: value,
        }
    }
}

fn serialize_recursive<SR, SS>(
    systems: &Systems,
    serialize_seq: &mut SS,
    _: PhantomData<SR>,
) -> Result<(), SS::Error>
where
    SS: SerializeSeq,
    SR: SystemRegistry,
{
    if SR::LEN == 0 {
        return Ok(())
    }

    if !systems.contains::<SR::Head>() {
        return Err(ser::Error::custom(format!(
            "the system {} was not found",
            short_type_name::<SR::Head>(),
        )));
    }

    #[cfg(any(test, debug_assertions))]
    debug!("Serializing the system {}", &short_type_name::<SR::Head>());
    // serialize_seq.serialize_element(TaggedRef::from())
    // serialize_seq.serialize_entry(
    //     &short_type_name::<SR::Head>(),
    //     &SerResource::new(&*resources.borrow::<RR::Head>()),
    // )?;

    serialize_recursive::<SR::Tail, SS>(systems, serialize_seq, PhantomData::default())
}

pub struct SerSystems<'a, SR> {
    systems: &'a Systems,
    _sr: PhantomData<SR>,
}

impl<'a, SR> From<&'a Systems> for SerSystems<'a, SR>
    where
        SR: SystemRegistry,
{
    fn from(systems: &'a Systems) -> Self {
        SerSystems {
            systems,
            _sr: PhantomData::default(),
        }
    }
}

impl<'a, SR> Serialize for SerSystems<'a, SR>
    where
        SR: SystemRegistry,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.systems.len()))?;

        serialize_recursive::<SR, S::SerializeSeq>(
            self.systems,
            &mut state,
            PhantomData::default(),
        )?;

        state.end()
    }
}

