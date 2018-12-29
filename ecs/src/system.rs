use crate::event::EventTrait;
use std::time::Duration;

/// Encodes a system or behaviour.
pub trait System<C> {
    /// Run the behaviour.
    fn run(&mut self, ctx: &mut C, t: &Duration, dt: &Duration);
}

/// Encodes a system or behaviour that processes the supplied event.
pub trait EventHandlerSystem<C, E>
where
    E: EventTrait,
{
    /// Returns the system's event filter, which selects the events that the system will expect.
    fn get_event_filter(&self) -> E::EventFlag;
    /// Run the behaviour.
    fn run(&mut self, ctx: &mut C, e: &E) -> bool;
}
