#![feature(try_from)]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[cfg(test)] #[macro_use] extern crate quickcheck;
#[cfg(test)] extern crate tempfile;
extern crate winit;
#[macro_use] extern crate ecs;

pub mod file_manipulation;
pub mod event;
pub mod auxiliary;
pub mod context;
pub mod systems;
pub mod orchestrator;
