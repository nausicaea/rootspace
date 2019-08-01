//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

#![warn(missing_docs)]

extern crate hibitset;
extern crate downcast_rs;

pub mod components;
pub mod entities;
pub mod events;
pub mod loop_stage;
pub mod persistence;
pub mod resources;
pub mod system;
pub mod world;

pub use crate::{
    components::{Component, Storage, VecStorage},
    entities::{Entities, Entity},
    events::{EventManager, EventTrait},
    loop_stage::LoopStage,
    persistence::Persistence,
    resources::{Resource, Resources},
    system::{EventHandlerSystem, System},
    world::{World, WorldTrait},
};
