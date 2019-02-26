//! Provides traits to specify behaviour (e.g. systems) that operates on data (e.g. components).

use std::time::Duration;
use crate::resources::Resources;
use crate::events::EventTrait;

/// Encodes a system or behaviour.
pub trait System {
    /// Run the behaviour.
    fn run(&mut self, res: &mut Resources, t: &Duration, dt: &Duration);
}

/// Encodes a system or behaviour that processes the supplied event.
pub trait EventHandlerSystem<E>
where
    E: EventTrait,
{
    /// Returns the system's event filter, which selects the events that the system will expect.
    fn get_event_filter(&self) -> E::EventFlag;
    /// Run the behaviour.
    fn run(&mut self, res: &mut Resources, e: &E) -> bool;
}
