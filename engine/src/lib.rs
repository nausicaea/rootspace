pub mod assets;
pub mod components;
pub mod debug_commands;
pub mod event;
pub mod geometry;
pub mod graphics;
pub mod orchestrator;
pub mod resources;
pub mod systems;
pub mod text_manipulation;
mod utilities;

pub const APP_QUALIFIER: &str = "net";
pub const APP_ORGANIZATION: &str = "nausicaea";

pub use self::{
    assets::{AssetMutTrait, AssetTrait},
    debug_commands::CommandTrait,
    event::EngineEvent,
    graphics::{glium::GliumBackend, headless::HeadlessBackend},
    orchestrator::{EmptyGame, Orchestrator},
};
