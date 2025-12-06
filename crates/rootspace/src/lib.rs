//#![warn(clippy::unwrap_used)]
#![recursion_limit = "256"]

pub mod app;
pub mod assets;
pub mod components;
pub mod events;
mod macros;
pub mod orchestrator;
mod registry;
pub mod resources;
pub mod systems;

pub use crate::{
    app::App,
    assets::scene::Scene,
    components::{camera::Camera, transform::Transform},
    orchestrator::{Orchestrator, OrchestratorDeps},
    resources::rpc_settings::RpcDeps,
    systems::rpc::service::RpcServiceClient,
};
pub use ecs::{
    Element, End,
    Resource,
    WithDependencies,
};
pub use griffon::components::renderable::RenderableSource;
