//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

pub use event_queue::receiver_id::ReceiverId;

pub use crate::{
    component::Component,
    entities::Entities,
    entity::Entity,
    event_queue::EventQueue,
    events::EventTrait,
    loop_control::LoopControl,
    loop_stage::LoopStage,
    registry::{Element, End, ResourceRegistry, SystemRegistry},
    resource::Resource,
    resources::Resources,
    serialization_name::SerializationName,
    short_type_name::short_type_name,
    storage::{Storage, vec_storage::VecStorage, zst_storage::ZstStorage},
    system::System,
    systems::Systems,
    with_resources::WithResources,
    world::{error::WorldError, event::WorldEvent, World},
};

pub mod component;
pub mod entities;
mod entity;
pub mod event_queue;
pub mod events;
mod loop_control;
pub mod loop_stage;
pub mod macros;
pub mod registry;
pub mod resource;
pub mod resources;
pub mod serialization_name;
pub mod short_type_name;
pub mod storage;
pub mod system;
pub mod systems;
pub mod with_resources;
pub mod world;
