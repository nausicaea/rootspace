//#![warn(clippy::unwrap_used)]

mod ecs;
mod engine;
mod file_manipulation;
mod forward_ref;
mod glamour;
mod plyers;
mod rose_tree;
mod urn;

pub use ecs::{
    registry::{Element, End},
    resource::Resource,
    with_dependencies::WithDependencies,
};
pub use engine::{
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
pub use glamour::{num::One, vec::Vec4};
pub use plyers::{
    load_ply, save_ply,
    types::{FormatType, PropertyDescriptor},
};
