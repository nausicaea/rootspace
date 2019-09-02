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

pub type DefaultOrchestrator<B, H> = Orchestrator<B, World<H>>;
pub type GliumEventInterface = EventInterface<GliumEventsLoop>;
pub type HeadlessEventInterface = EventInterface<HeadlessEventsLoop>;
pub type GliumRenderer = Renderer<GliumBackend>;
pub type HeadlessRenderer = Renderer<HeadlessBackend>;
