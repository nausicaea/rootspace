use super::{
    super::{entities::Entities, event_queue::EventQueue},
    event::WorldEvent,
};
use crate::RegAdd;

/// Prepends the [`Resource`s](crate::resource::Resource) [`Entities`](crate::entities::Entities)
/// and [`EventQueue<T>`](crate::event_queue::EventQueue) to externally defined resources in a
/// heterogeneous list that implements [`ResourceRegistry`](crate::registry::ResourceRegistry).
pub type ResourceTypes<RR> = RegAdd![Entities, EventQueue<WorldEvent>, RR];
