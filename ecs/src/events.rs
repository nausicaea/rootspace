//! Provides facilities to define and manage events.

use crate::resources::Resource;
use std::{
    collections::VecDeque,
    fmt,
    ops::{BitAnd, BitOr, BitXor},
};

/// Events sent around within the `World` need to implement this trait such that individual
/// `EventHandlerSystem`s may filter for particular events.
pub trait EventTrait: Clone + fmt::Debug + 'static {
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

/// An `EventManager` contains a queue of events and provides rudimentary facilities of retrieving
/// those events.
#[cfg_attr(feature = "diagnostics", derive(TypeName))]
pub struct EventManager<E>(VecDeque<E>);

impl<E> EventManager<E> {
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

impl<E> Default for EventManager<E> {
    fn default() -> Self {
        EventManager(VecDeque::default())
    }
}

impl<E> fmt::Debug for EventManager<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventManager {{ ... }}")
    }
}

impl<E> Resource for EventManager<E> where E: 'static {}
