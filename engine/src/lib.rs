#![feature(try_from)]

#[cfg(test)] #[macro_use] extern crate assertions;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate failure;
extern crate hierarchy;
extern crate glium;
#[macro_use] extern crate log;
#[cfg(test)] #[macro_use] extern crate quickcheck;
#[cfg(test)] extern crate tempfile;
#[macro_use] extern crate ecs;

//pub mod auxiliary;
pub mod components;
pub mod context;
pub mod event;
pub mod file_manipulation;
pub mod math;
pub mod mock;
pub mod orchestrator;
pub mod systems;
pub mod wrappers;
