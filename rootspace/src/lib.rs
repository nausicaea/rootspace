//#![warn(clippy::unwrap_used)]
#![recursion_limit = "256"]

pub mod assets;
pub mod components;
pub mod events;
mod macros;
pub mod orchestrator;
mod registry;
pub mod resources;
pub mod systems;

pub use ecs::{
    registry::{Element, End},
    resource::Resource,
    with_dependencies::WithDependencies,
};
pub use crate::{
    assets::scene::{RenderableSource, Scene},
    components::{camera::Camera, transform::Transform},
    orchestrator::{Orchestrator, OrchestratorDeps},
    resources::{
        asset_database::{AssetDatabase, AssetDatabaseDeps},
        graphics::{settings::Settings, GraphicsDeps},
        rpc_settings::RpcDeps,
    },
    systems::rpc::service::RpcServiceClient,
};