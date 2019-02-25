//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

extern crate hibitset;
#[macro_use]
extern crate mopa;

pub mod loop_stage;
pub mod entities;
pub mod components;
pub mod events;
pub mod resources;
pub mod system;
pub mod world;

pub use crate::{
    events::{EventTrait, EventManager},
    entities::{Entity, Entities},
    components::VecStorage,
    resources::{Resource, Resources},
    loop_stage::LoopStage,
    system::{System, EventHandlerSystem},
    world::{World, WorldTrait},
};
