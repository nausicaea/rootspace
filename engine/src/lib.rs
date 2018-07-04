#![feature(try_from)]

#[cfg(test)]
#[macro_use]
extern crate approx;
#[cfg(test)]
#[macro_use]
extern crate assertions;
extern crate affine_transform;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate glium;
extern crate hierarchy;
#[macro_use]
extern crate log;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate nalgebra;
#[cfg(test)]
extern crate tempfile;
#[macro_use]
extern crate ecs;

pub mod components;
pub mod context;
pub mod event;
pub mod file_manipulation;
pub mod graphics;
#[cfg(any(test, feature = "mock"))]
pub mod mock;
pub mod orchestrator;
pub mod systems;
