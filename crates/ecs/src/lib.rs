//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

mod component;
mod entities;
mod entity;
mod event_monitor;
mod event_queue;
mod loop_control;
mod macros;
mod registry;
mod resource;
mod resources;
mod storage;
mod system;
mod systems;
mod with_dependencies;
mod with_resources;
mod world;

pub use crate::{
    component::Component,
    entities::Entities,
    entity::Entity,
    entity::index::Index,
    event_monitor::EventMonitor,
    event_queue::{EventQueue, receiver_id::ReceiverId},
    loop_control::LoopControl,
    registry::{Element, End, ResourceRegistry, SystemRegistry},
    resource::Resource,
    resources::Resources,
    storage::{Storage, vec_storage::VecStorage, zst_storage::ZstStorage},
    system::System,
    with_dependencies::WithDependencies,
    with_resources::WithResources,
    world::{World, event::WorldEvent},
};
