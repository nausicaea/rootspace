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
        glium::GliumBackend,
        headless::HeadlessBackend,
    },
    orchestrator::{JoinedRegistry, Orchestrator},
    systems::{event_interface::EventInterface, renderer::Renderer},
};
use ecs::World;

pub type DefaultOrchestrator<B, RR> = Orchestrator<B, RR, World<JoinedRegistry<RR>>>;
pub type GliumEventInterface = EventInterface<GliumBackend>;
pub type HeadlessEventInterface = EventInterface<HeadlessBackend>;
pub type GliumRenderer = Renderer<GliumBackend>;
pub type HeadlessRenderer = Renderer<HeadlessBackend>;
