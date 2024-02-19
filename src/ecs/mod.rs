//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

pub mod component;
pub mod entities;
pub mod entity;
pub mod event_monitor;
pub mod event_queue;
pub mod loop_control;
pub mod loop_stage;
pub mod macros;
pub mod registry;
pub mod resource;
pub mod resources;
pub mod storage;
pub mod system;
pub mod systems;
pub mod with_dependencies;
pub mod with_resources;
pub mod world;
