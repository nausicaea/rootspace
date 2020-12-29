use crate::{Resources, ResourceRegistry, Resource};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, de};
use std::marker::PhantomData;
use std::fmt;
use log::debug;

fn deserialize_entry<'de, A, RR, R>(
    resources: &mut Resources,
    map_access: &mut A,
    type_name: &str,
    _: PhantomData<RR>,
    _: PhantomData<R>,
) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        RR: ResourceRegistry,
        R: Resource + Deserialize<'de>,
{
    if type_name == std::any::type_name::<R>() {
        let c = map_access.next_value::<R>()?;
        resources.insert(c);
        Ok(())
    } else {
        deserialize_recursive::<A, RR::Tail>(
            resources,
            map_access,
            type_name,
            PhantomData::default(),
        )
    }
}

fn deserialize_recursive<'de, A, RR>(
    resources: &mut Resources,
    map_access: &mut A,
    type_name: &str,
    _: PhantomData<RR>,
) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
        RR: ResourceRegistry,
{

    if RR::LEN > 0 {
        deserialize_entry::<A, RR, RR::Head>(
            resources,
            map_access,
            type_name,
            PhantomData::default(),
            PhantomData::default(),
        )
    } else {
        Err(de::Error::custom(format!("Not a registered type: {}", type_name)))
    }
}

pub struct ResourcesVisitor<RR>(PhantomData<RR>);

impl<RR> Default for ResourcesVisitor<RR> {
    fn default() -> Self {
        ResourcesVisitor(PhantomData::default())
    }
}

impl<RR> std::fmt::Debug for ResourcesVisitor<RR> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ResourcesVisitor<{0}>(PhantomData<{0}>)", std::any::type_name::<RR>())
    }
}

impl<'de, RR> Visitor<'de> for ResourcesVisitor<RR>
    where
        RR: ResourceRegistry,
{
    type Value = Resources;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a map of type names to their serialized data"
        )
    }

    fn visit_map<A>(self, mut map_access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
    {
        let capacity = map_access.size_hint().unwrap_or(RR::LEN);
        let mut resources = Resources::with_capacity(capacity);

        while let Some(type_name) = map_access.next_key::<String>()? {
            #[cfg(any(test, debug_assertions))]
            debug!("Deserializing the resource {}", &type_name);
            deserialize_recursive::<A, RR>(
                &mut resources,
                &mut map_access,
                &type_name,
                PhantomData::default(),
            )?;
        }

        // FIXME: Null-pointer deref when dropping a VecStorage<T> (appears in vec_storage.rs:96 in `self.index.iter()`)
        Ok(resources)
    }
}

#[cfg(test)]
mod tests {
    use super::ResourcesVisitor;
    use crate::Reg;
    use serde::de::Deserializer;
    use crate::{VecStorage, Entities};

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Error(\"Not a registered type: BogusType\", line: 1, column: 12)")]
    fn test_de_with_entities() {
        pub type TestRegistry = Reg![
            Entities,
        ];

        let visitor = ResourcesVisitor::<TestRegistry>::default();
        let mut d = serde_json::Deserializer::from_str("{\"BogusType\":null}");
        let _resources = d.deserialize_map(visitor).unwrap();
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Error(\"Not a registered type: BogusType\", line: 1, column: 12)")]
    fn test_de_with_vec_storage() {
        pub type TestRegistry = Reg![
            VecStorage<usize>,
        ];

        let visitor = ResourcesVisitor::<TestRegistry>::default();
        let mut d = serde_json::Deserializer::from_str("{\"BogusType\":null}");
        let _resources = d.deserialize_map(visitor).unwrap();
    }
}