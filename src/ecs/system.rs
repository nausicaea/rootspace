//! Provides traits to specify behaviour (e.g. systems) that operates on data (e.g. components).

use async_trait::async_trait;
use std::time::Duration;

use super::resources::Resources;

/// Encodes a system or behaviour.
#[async_trait]
pub trait System: 'static + Sync + Send {
    /// Return the system's name.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Run the behaviour.
    async fn run(&mut self, res: &Resources, t: Duration, dt: Duration);
}

#[async_trait]
impl System for () {
    async fn run(&mut self, _: &Resources, _: Duration, _: Duration) {}
}
