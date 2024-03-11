use crate::ecs::component::Component;
use crate::ecs::entity::index::Index;
use crate::ecs::event_monitor::EventMonitor;
use crate::ecs::event_queue::EventQueue;
use crate::ecs::world::event::WorldEvent;
use crate::engine::components::camera::Camera;
use crate::engine::components::info::Info;
use crate::engine::components::renderable::Renderable;
use crate::engine::components::status::Status;
use crate::engine::components::transform::Transform;
use crate::engine::components::ui_transform::UiTransform;
use crate::engine::events::engine_event::EngineEvent;
use crate::engine::resources::asset_database::AssetDatabase;
use crate::engine::resources::graphics::Graphics;
use crate::engine::resources::rpc_settings::RpcSettings;
use crate::engine::resources::statistics::Statistics;
use crate::engine::systems::camera_manager::CameraManager;
use crate::engine::systems::force_shutdown::ForceShutdown;
use crate::engine::systems::rpc::Rpc;
use crate::rose_tree::hierarchy::Hierarchy;
use crate::RegAdd;
use winit::event::WindowEvent;

use super::systems::debug_animator::DebugAnimator;

pub type RRegistry<S> = RegAdd![
    <Camera as Component>::Storage,
    <Info as Component>::Storage,
    <Transform as Component>::Storage,
    <Renderable as Component>::Storage,
    <Status as Component>::Storage,
    <UiTransform as Component>::Storage,
    AssetDatabase,
    EventQueue<WindowEvent>,
    EventQueue<EngineEvent>,
    Graphics,
    Hierarchy<Index>,
    Statistics,
    RpcSettings,
    S
];

pub type FUSRegistry<D> = RegAdd![DebugAnimator, D];

pub type USRegistry<D> = RegAdd![
    CameraManager,
    ForceShutdown,
    EventMonitor<WindowEvent>,
    EventMonitor<EngineEvent>,
    EventMonitor<WorldEvent>,
    D
];

pub type MSRegistry<D> = RegAdd![Rpc, D];
