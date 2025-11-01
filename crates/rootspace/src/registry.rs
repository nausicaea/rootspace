use winit::event::WindowEvent;

use super::systems::debug_animator::DebugAnimator;
use assam::AssetDatabase;
use griffon::Graphics;
use crate::{
    components::{
        camera::Camera, debug_animate::DebugAnimate, info::Info, light::Light, renderable::Renderable,
        transform::Transform,
    },
    events::engine_event::EngineEvent,
    resources::{rpc_settings::RpcSettings, statistics::Statistics},
    systems::{
        camera_controller::CameraController, camera_manager::CameraManager, force_shutdown::ForceShutdown, rpc::Rpc,
    },
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
    <Light as Component>::Storage,
    <DebugAnimate as Component>::Storage,
    AssetDatabase,
    EventQueue<WindowEvent>,
    EventQueue<EngineEvent>,
    Graphics,
    Hierarchy<Index>,
    Statistics,
    RpcSettings,
    S
];

pub type FUSRegistry<D> = RegAdd![DebugAnimator, CameraController, D];

pub type USRegistry<D> = RegAdd![
    CameraManager,
    ForceShutdown,
    EventMonitor<WindowEvent>,
    EventMonitor<EngineEvent>,
    EventMonitor<WorldEvent>,
    D
];

pub type MSRegistry<D> = RegAdd![Rpc, D];
