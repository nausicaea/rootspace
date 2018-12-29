//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

#[cfg(test)]
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate snowflake;

pub mod database;
pub mod entity;
pub mod event;
pub mod loop_stage;
#[cfg(any(test, feature = "mock"))]
pub mod mock;
pub mod system;
pub mod world;

pub use crate::{
    database::{Database, DatabaseTrait, Error as DatabaseError},
    entity::Entity,
    event::{EventManagerTrait, EventTrait},
    loop_stage::LoopStage,
    world::{World, WorldTrait},
};
