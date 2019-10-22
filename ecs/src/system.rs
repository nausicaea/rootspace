//! Provides traits to specify behaviour (e.g. systems) that operates on data (e.g. components).

use crate::resources::Resources;
use downcast_rs::{impl_downcast, Downcast};
use std::time::Duration;

/// Encodes a system or behaviour.
pub trait System: Downcast {
    /// Return the system's name.
    fn name(&self) -> &'static str;
    /// Run the behaviour.
    fn run(&mut self, res: &Resources, t: &Duration, dt: &Duration);
}

impl_downcast!(System);

impl System for () {
    fn name(&self) -> &'static str {
        "()"
    }

    fn run(&mut self, _: &Resources, _: &Duration, _: &Duration) {}
}

pub struct Systems(Vec<Box<dyn System>>);

impl Systems {
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
        self.0
            .iter()
            .filter_map(|s| s.downcast_ref::<S>())
            .last()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn System>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn System>> {
        self.0.iter_mut()
    }
}

impl Default for Systems {
    fn default() -> Self {
        Systems(Vec::default())
    }
}
