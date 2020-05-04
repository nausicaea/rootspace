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
        stringify!(())
    }

    fn run(&mut self, _: &Resources, _: &Duration, _: &Duration) {}
}
