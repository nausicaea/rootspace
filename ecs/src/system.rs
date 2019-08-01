//! Provides traits to specify behaviour (e.g. systems) that operates on data (e.g. components).

use crate::{events::EventTrait, resources::Resources};
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

/// Encodes a system or behaviour that processes the supplied event.
pub trait EventHandlerSystem<E>: Downcast
where
    E: EventTrait,
{
    /// Return the system's name.
    fn name(&self) -> &'static str;
    /// Return the system's event filter, which selects the events that the system will expect.
    fn get_event_filter(&self) -> E::EventFlag;
    /// Run the behaviour.
    fn run(&mut self, res: &mut Resources, e: &E) -> bool;
}

impl_downcast!(EventHandlerSystem<E> where E: EventTrait);
