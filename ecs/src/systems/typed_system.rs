use std::marker::PhantomData;
use either::Either;
use either::Either::{Left, Right};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;

use crate::system::System;

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum TypedSystemField {
    Order,
    System,
}

const TYPED_SYSTEM_FIELDS: &'static [&'static str] = &[
    "order",
    "system",
];

#[derive(Debug)]
pub struct TypedSystem<'a, S> {
    pub order: usize,
    pub system: Either<&'a S, S>,
}

impl<'a, S> TypedSystem<'a, S>
where
    S: System + Serialize,
{
    pub fn new(order: usize, system: &'a S) -> Self {
        TypedSystem {
            order,
            system: Left(system),
        }
    }
}

impl<'a, S> Serialize for TypedSystem<'a, S>
where
    S: System + Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let mut state = serializer.serialize_struct("TypedSystem", 2)?;
        state.serialize_field("order", &self.order)?;
        state.serialize_field("system", self.system.as_ref().either(|&ref_s| ref_s, |s| s))?;
        state.end()
    }
}

impl<'de, 'a, S> Deserialize<'de> for TypedSystem<'a, S>
where
    S: System + Deserialize<'de>,
{
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: Deserializer<'de>,
    {
        deserializer.deserialize_struct("TypedSystem", TYPED_SYSTEM_FIELDS, TypedSystemVisitor::default())
    }
}

#[derive(Debug)]
struct TypedSystemVisitor<'a, S>(PhantomData<&'a S>);

impl<'a, S> Default for TypedSystemVisitor<'a, S>
where
    S: System,
{
    fn default() -> Self {
        TypedSystemVisitor(PhantomData::default())
    }
}

impl<'de, 'a, S> Visitor<'de> for TypedSystemVisitor<'a, S>
where
    S: System + Deserialize<'de>,
{
    type Value = TypedSystem<'a, S>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a serialized system")
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut order: Option<usize> = None;
        let mut system: Option<S> = None;

        while let Some(field_name) = map_access.next_key()? {
            match field_name {
                TypedSystemField::Order => {
                    if order.is_some() {
                        return Err(de::Error::duplicate_field("order"));
                    }
                    order = Some(map_access.next_value()?);
                },
                TypedSystemField::System => {
                    if system.is_some() {
                        return Err(de::Error::duplicate_field("system"));
                    }
                    system = Some(map_access.next_value()?);
                },
            }
        }

        let order = order.ok_or_else(|| de::Error::missing_field("order"))?;
        let system = system.ok_or_else(|| de::Error::missing_field("system"))?;

        Ok(TypedSystem {
            order,
            system: Right(system),
        })
    }
}
