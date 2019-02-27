extern crate alga;
#[cfg(test)]
#[macro_use]
extern crate approx;
extern crate affine_transform;
#[cfg(any(test, feature = "mock"))]
#[macro_use]
extern crate bitflags;
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
#[macro_use]
extern crate quickcheck;
extern crate ecs;
extern crate nalgebra;
extern crate num_traits;
extern crate ply;
extern crate rusttype;
#[cfg(test)]
extern crate tempfile;
extern crate unicode_normalization;

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
pub mod scene_graph;
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

pub type DefaultOrchestrator<E> = Orchestrator<E, World<E>>;
pub type GliumEventInterface<E> = EventInterface<E, GliumEventsLoop>;
pub type HeadlessEventInterface<E> = EventInterface<E, HeadlessEventsLoop>;
pub type GliumRenderer<E> = Renderer<E, GliumBackend>;
pub type HeadlessRenderer<E> = Renderer<E, HeadlessBackend>;
