pub(crate) mod serialization;
pub(crate) mod deserialization;

use crate::system::System;
use std::slice::{Iter, IterMut};

pub struct Systems(Vec<Box<dyn System>>);

impl Systems {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn contains<S>(&self) -> bool
    where
        S: System,
    {
        self.0.iter()
            .any(|s| s.is::<S>())
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

    pub fn find<S>(&self) -> Option<&S>
    where
        S: System,
    {
        self.0.iter().filter_map(|s| s.downcast_ref::<S>()).nth(0)
    }

    pub fn find_mut<S>(&mut self) -> Option<&mut S>
    where
        S: System,
    {
        self.0
            .iter_mut()
            .filter_map(|s| s.downcast_mut::<S>())
            .last()
    }

    pub fn iter(&self) -> Iter<'_, Box<dyn System>> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Box<dyn System>> {
        self.into_iter()
    }
}

impl Default for Systems {
    fn default() -> Self {
        Systems(Vec::default())
    }
}

impl<'a> IntoIterator for &'a Systems {
    type Item = &'a Box<dyn System>;
    type IntoIter = Iter<'a, Box<dyn System>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Systems {
    type Item = &'a mut Box<dyn System>;
    type IntoIter = IterMut<'a, Box<dyn System>>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}