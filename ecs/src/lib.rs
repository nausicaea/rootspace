//! The `ecs` library provides facilities to build an Entity-Component-System architecture, wherein
//! Entities serve as identifiers, Components contain (mostly) data and are always in a
//! relationship with Entities, and finally, Systems encode (mostly) behaviour. The World manages
//! all three type categories and provides access to each.

#[cfg(test)]
#[macro_use]
extern crate assertions;
extern crate log;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate bitflags;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate snowflake;

pub mod database;
pub mod entity;
pub mod event;
pub mod loop_stage;
pub mod macros;
pub mod mock;
pub mod system;
pub mod world;

pub use self::database::{Database, DatabaseTrait, Error as DatabaseError};
pub use self::entity::Entity;
pub use self::event::{EventManagerTrait, EventTrait};
pub use self::loop_stage::LoopStage;
pub use self::system::SystemTrait;
pub use self::world::{World, WorldTrait};
