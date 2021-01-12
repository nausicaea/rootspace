use crate::registry::SystemRegistry;
use super::Systems;
use std::marker::PhantomData;
use serde::de::{Deserialize, Deserializer};

pub struct DeSystems<SR> {
    systems: Systems,
    _sr: PhantomData<SR>,
}

impl<SR> From<DeSystems<SR>> for Systems
    where
        SR: SystemRegistry,
{
    fn from(value: DeSystems<SR>) -> Self {
        value.systems
    }
}

impl<'de, SR> Deserialize<'de> for DeSystems<SR>
    where
        SR: SystemRegistry,
{
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        todo!()
    }
}
