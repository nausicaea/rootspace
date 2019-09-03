//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

//#![warn(missing_docs)]

pub mod components;
pub mod entities;
pub mod event_queue;
pub mod events;
mod indexing;
pub mod loop_stage;
pub mod macros;
pub mod persistence;
pub mod registry;
pub mod resource;
pub mod resources;
pub mod system;
pub mod world;

pub use crate::{
    components::{Component, Storage, VecStorage},
    entities::{Entities, Entity},
    event_queue::{EventQueue, ReceiverId},
    events::EventTrait,
    loop_stage::LoopStage,
    persistence::Persistence,
    registry::{Registry, Element, End},
    resource::Resource,
    resources::Resources,
    system::System,
    world::{World, WorldEvent, ResourcesTrait, WorldTrait},
};
