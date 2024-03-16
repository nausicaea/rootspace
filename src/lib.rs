//#![warn(clippy::unwrap_used)]

mod ecs;
mod engine;
mod file_manipulation;
mod forward_ref;
mod glamour;
mod plyers;
mod rose_tree;
mod urn;

pub use engine::orchestrator::{Orchestrator, OrchestratorDeps};
pub use engine::resources::asset_database::{AssetDatabaseDeps, AssetDatabase};
pub use engine::resources::graphics::settings::Settings;
pub use engine::resources::graphics::GraphicsDeps;
pub use engine::resources::rpc_settings::RpcDeps;
pub use engine::systems::rpc::service::RpcServiceClient;
pub use ecs::registry::End;
pub use ecs::registry::Element;
pub use ecs::resource::Resource;
pub use ecs::with_dependencies::WithDependencies;
pub use engine::assets::scene::RenderableSource;
pub use engine::assets::scene::Scene;
pub use engine::components::camera::Camera;
pub use engine::components::transform::Transform;
pub use plyers::types::{FormatType, PropertyDescriptor};
pub use plyers::{load_ply, save_ply};
pub use glamour::num::One;
pub use glamour::vec::Vec4;