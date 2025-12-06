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
    resource::Resource,
    resources::Resources,
    with_dependencies::WithDependencies,
    registry::{Element, End, SystemRegistry, ResourceRegistry},
    event_monitor::EventMonitor,
    component::Component,
    storage::{
        Storage,
        vec_storage::VecStorage,
        zst_storage::ZstStorage
    },
    entity::Entity,
    loop_control::LoopControl,
    event_queue::{EventQueue, receiver_id::ReceiverId},
    system::System,
    with_resources::WithResources,
    world::{
        World,
        event::WorldEvent,
    },
    entities::Entities,
    entity::index::Index,
};
