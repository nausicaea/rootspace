//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

pub mod components;
pub mod entities;
pub mod event_queue;
pub mod events;
//mod hibitset;
mod indexing;
pub mod loop_stage;
pub mod macros;
pub mod registry;
pub mod resource;
pub mod resources;
pub mod system;
pub mod systems;
pub mod world;

pub use crate::{
    components::{Component, Storage, VecStorage, ZstStorage},
    entities::{Entities, Entity},
    event_queue::{EventQueue, ReceiverId},
    events::EventTrait,
    loop_stage::LoopStage,
    registry::{Element, End, ResourceRegistry},
    resource::Resource,
    resources::{Persistence, Resources, Settings},
    system::System,
    systems::Systems,
    world::{ResourcesTrait, World, WorldEvent, WorldTrait},
};
