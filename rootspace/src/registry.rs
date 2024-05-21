use winit::event::WindowEvent;

use super::systems::debug_animator::DebugAnimator;
use crate::{
    components::{camera::Camera, info::Info, renderable::Renderable, transform::Transform},
    events::engine_event::EngineEvent,
    resources::{asset_database::AssetDatabase, graphics::Graphics, rpc_settings::RpcSettings, statistics::Statistics},
    systems::{camera_manager::CameraManager, force_shutdown::ForceShutdown, rpc::Rpc},
};
use ecs::{
    component::Component, entity::index::Index, event_monitor::EventMonitor, event_queue::EventQueue,
    world::event::WorldEvent, RegAdd,
};
use rose_tree::hierarchy::Hierarchy;

pub type RRegistry<S> = RegAdd![
    <Camera as Component>::Storage,
    <Info as Component>::Storage,
    <Transform as Component>::Storage,
    <Renderable as Component>::Storage,
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
