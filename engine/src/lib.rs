#[cfg(test)]
#[macro_use]
extern crate approx;
extern crate affine_transform;
extern crate clap;
extern crate ctrlc;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate glium;
extern crate hierarchy;
extern crate image;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
extern crate ecs;
extern crate nalgebra;
extern crate num_traits;
extern crate ply;
extern crate rusttype;
extern crate snowflake;
#[cfg(test)]
extern crate tempfile;
extern crate unicode_normalization;
#[cfg(feature = "diagnostics")]
#[macro_use]
extern crate typename;

pub mod assets;
pub mod components;
pub mod debug_commands;
pub mod event;
pub mod file_manipulation;
pub mod geometry;
pub mod graphics;
#[cfg(any(test, feature = "mock"))]
pub mod mock;
pub mod orchestrator;
pub mod resources;
pub mod systems;
pub mod text_manipulation;

use crate::{
    graphics::{
        glium::{GliumBackend, GliumEventsLoop},
        headless::{HeadlessBackend, HeadlessEventsLoop},
    },
    orchestrator::Orchestrator,
    systems::{event_interface::EventInterface, renderer::Renderer},
};
use ecs::World;

pub type DefaultOrchestrator<B> = Orchestrator<B, World>;
pub type GliumEventInterface = EventInterface<GliumEventsLoop>;
pub type HeadlessEventInterface = EventInterface<HeadlessEventsLoop>;
pub type GliumRenderer = Renderer<GliumBackend>;
pub type HeadlessRenderer = Renderer<HeadlessBackend>;
