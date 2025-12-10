use super::{
    super::{entities::Entities, event_queue::EventQueue},
    event::WorldEvent,
};
use crate::RegAdd;

/// Prepends the [`Resource`]s [`Entities`]
/// and [`EventQueue`] to externally defined resources in a
/// heterogeneous list that implements [`crate::registry::ResourceRegistry`]
pub type ResourceTypes<RR> = RegAdd![Entities, EventQueue<WorldEvent>, RR];
