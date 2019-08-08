//! Provides traits to specify behaviour (e.g. systems) that operates on data (e.g. components).

use crate::resources::Resources;
use downcast_rs::{impl_downcast, Downcast};
use std::time::Duration;

/// Encodes a system or behaviour.
pub trait System: Downcast {
    /// Return the system's name.
    fn name(&self) -> &'static str;
    /// Run the behaviour.
    fn run(&mut self, res: &mut Resources, t: &Duration, dt: &Duration);
}

impl_downcast!(System);
