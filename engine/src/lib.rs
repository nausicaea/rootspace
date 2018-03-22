extern crate log;
#[macro_use] extern crate failure;
extern crate ecs;
#[cfg(test)] extern crate tempfile;
#[cfg(test)] #[macro_use] extern crate quickcheck;

pub mod file_manipulation;
pub mod orchestrator;
