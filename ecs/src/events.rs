//! Provides facilities to define and manage events.

use std::{
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
