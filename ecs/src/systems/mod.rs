mod recursors;
mod typed_system;
pub(crate) mod typed_systems;

use std::slice::{Iter, IterMut};

use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};

use self::typed_systems::TypedSystems;
use crate::{registry::SystemRegistry, resources::Resources, system::System, with_resources::WithResources};

#[derive(Default)]
pub struct Systems(Vec<Box<dyn System>>);

impl std::fmt::Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "System(#{})", self.0.len())
    }
}

impl Systems {
    pub fn with_capacity(cap: usize) -> Self {
        Systems(Vec::with_capacity(cap))
    }

    pub fn with_registry<SR>(res: &Resources) -> Self
    where
        SR: SystemRegistry,
    {
        let helper = TypedSystems::<SR>::with_resources(res);
        Systems::from(helper)
    }

    pub fn deserialize_with<'de, SR, D>(deserializer: D) -> Result<Self, D::Error>
    where
        SR: SystemRegistry,
        D: Deserializer<'de>,
    {
        let helper = TypedSystems::<SR>::deserialize(deserializer)?;
        Ok(Systems::from(helper))
    }

    pub fn serialize_with<SR, S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        SR: SystemRegistry,
        S: Serializer,
    {
        let status = TypedSystems::<SR>::from(self).serialize(serializer)?;
        Ok(status)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains<S>(&self) -> bool
    where
        S: System,
    {
        self.0.iter().any(|s| s.is::<S>())
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn insert<S>(&mut self, sys: S)
    where
        S: System,
    {
        self.0.push(Box::new(sys))
    }

    pub fn get<S>(&self) -> &S
    where
        S: System,
    {
        self.find::<S>()
            .unwrap_or_else(|| panic!("Could not find the system {}", std::any::type_name::<S>()))
    }

    pub fn get_mut<S>(&mut self) -> &mut S
    where
        S: System,
    {
        self.find_mut::<S>()
            .unwrap_or_else(|| panic!("Could not find the system {}", std::any::type_name::<S>()))
    }

    pub fn find_with_position<S>(&self) -> Option<(usize, &S)>
    where
        S: System,
    {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.downcast_ref::<S>().map(|sdc| (i, sdc)))
            .next()
    }

    pub fn find<S>(&self) -> Option<&S>
    where
        S: System,
    {
        self.0.iter().filter_map(|s| s.downcast_ref::<S>()).next()
    }

    pub fn find_mut<S>(&mut self) -> Option<&mut S>
    where
        S: System,
    {
        self.0.iter_mut().filter_map(|s| s.downcast_mut::<S>()).last()
    }

    pub fn iter(&self) -> Iter<'_, Box<dyn System>> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Box<dyn System>> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a Systems {
    type IntoIter = Iter<'a, Box<dyn System>>;
    type Item = &'a Box<dyn System>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).iter()
    }
}

impl<'a> IntoIterator for &'a mut Systems {
    type IntoIter = IterMut<'a, Box<dyn System>>;
    type Item = &'a mut Box<dyn System>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).iter_mut()
    }
}

impl PartialEq for Systems {
    fn eq(&self, rhs: &Self) -> bool {
        if self.len() != rhs.len() {
            return false;
        }

        self.0.iter().zip(rhs).all(|(lhs, rhs)| lhs.name() == rhs.name())
    }
}
