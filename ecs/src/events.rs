//! Provides facilities to define and manage events.

use crate::resources::Resource;
use std::{
    collections::VecDeque,
    fmt,
    ops::{BitAnd, BitOr, BitXor},
};

/// Events sent around within the `World` need to implement this trait such that individual
/// `EventHandlerSystem`s may filter for particular events.
pub trait EventTrait: Clone + 'static {
    /// Defines the event filter type.
    type EventFlag: Default
        + Clone
        + Copy
        + PartialEq
        + BitAnd<Output = Self::EventFlag>
        + BitOr<Output = Self::EventFlag>
        + BitXor<Output = Self::EventFlag>;

    /// Given an event filter, returns `true` if the current event matches that filter, `false`
    /// otherwise.
    ///
    /// # Arguments
    ///
    /// * `flag` - The filter (or bitflag) with which to evaluate the current event.
    fn matches_filter(&self, flag: Self::EventFlag) -> bool;
}

/// An `EventQueue` contains a queue of events and provides rudimentary facilities of retrieving
/// those events.
#[cfg_attr(feature = "diagnostics", derive(TypeName))]
pub struct EventQueue<E>(VecDeque<E>);

impl<E> EventQueue<E> {
    /// Dispatches an event to the queue.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to add to the queue.
    pub fn dispatch_later(&mut self, event: E) {
        self.0.push_back(event)
    }

    /// Returns the length of the event queue (e.g. the number of queued events).
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Empties the queue and returns all queued events in FIFO (first-in, first-out) order.
    pub fn flush(&mut self) -> Vec<E> {
        self.0.drain(..).collect()
    }
}

impl<E> Default for EventQueue<E> {
    fn default() -> Self {
        EventQueue(VecDeque::default())
    }
}

impl<E> fmt::Debug for EventQueue<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventQueue {{ #events: {} }}", self.0.len())
    }
}

impl<E> Resource for EventQueue<E> where E: 'static {}
