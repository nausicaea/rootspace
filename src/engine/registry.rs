use winit::event::WindowEvent;

use super::systems::debug_animator::DebugAnimator;
use crate::{
    ecs::{
        component::Component, entity::index::Index, event_monitor::EventMonitor, event_queue::EventQueue,
        world::event::WorldEvent,
    },
    engine::{
        components::{
            camera::Camera, info::Info, renderable::Renderable, transform::Transform, ui_transform::UiTransform,
        },
        events::engine_event::EngineEvent,
        resources::{
            asset_database::AssetDatabase, graphics::Graphics, rpc_settings::RpcSettings, statistics::Statistics,
        },
        systems::{camera_manager::CameraManager, force_shutdown::ForceShutdown, rpc::Rpc},
    },
    rose_tree::hierarchy::Hierarchy,
    RegAdd,
};

pub type RRegistry<S> = RegAdd![
    <Camera as Component>::Storage,
    <Info as Component>::Storage,
    <Transform as Component>::Storage,
    <Renderable as Component>::Storage,
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
