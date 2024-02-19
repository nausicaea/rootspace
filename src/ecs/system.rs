//! Provides traits to specify behaviour (e.g. systems) that operates on data (e.g. components).

use std::time::Duration;

use downcast_rs::{impl_downcast, Downcast};

use super::resources::Resources;

/// Encodes a system or behaviour.
pub trait System: Downcast {
    /// Return the system's name.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Run the behaviour.
    fn run(&mut self, res: &Resources, t: &Duration, dt: &Duration);
}

impl_downcast!(System);

impl System for () {
    fn run(&mut self, _: &Resources, _: &Duration, _: &Duration) {}
}
