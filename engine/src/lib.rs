#[cfg(test)]
#[macro_use]
extern crate approx;
#[cfg(test)]
#[macro_use]
extern crate assertions;
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
pub mod context;
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

use context::Context;
use ecs::World;
use graphics::{
    glium::{GliumBackend, GliumEventsLoop},
    headless::{HeadlessBackend, HeadlessEventsLoop},
};
use orchestrator::Orchestrator;
use systems::{EventInterface, Renderer, SystemGroup};

pub type DefaultWorld<E> = World<E, Context<E>, SystemGroup<Context<E>, E>>;
pub type DefaultOrchestrator<E> = Orchestrator<DefaultWorld<E>>;
pub type GliumEventInterface<C, E> = EventInterface<C, E, GliumEventsLoop>;
pub type HeadlessEventInterface<C, E> = EventInterface<C, E, HeadlessEventsLoop>;
pub type GliumRenderer<C, E> = Renderer<C, E, GliumBackend>;
pub type HeadlessRenderer<C, E> = Renderer<C, E, HeadlessBackend>;
