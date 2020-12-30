use super::event::WorldEvent;
use crate::{Entities, EventQueue, RegAdd};

/// Prepends the [`Resource`s](crate::resource::Resource) [`Entities`](crate::entities::Entities)
/// and [`EventQueue<T>`](crate::event_queue::EventQueue) to externally defined resources in a
/// heterogeneous list that implements [`ResourceRegistry`](crate::registry::ResourceRegistry).
pub type JoinedRegistry<RR> = RegAdd![Entities, EventQueue<WorldEvent>, RR];
