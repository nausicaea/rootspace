use ecs::Component;
use ecs::EventQueue;
use ecs::RegAdd;
use ecs::WorldEvent;

use crate::components::{Camera, Info, Model, Renderable, Status, UiModel};
use crate::event::EngineEvent;
use crate::resources::SceneGraph;
use crate::systems::{
    camera_manager::CameraManager,
    debug_console::DebugConsole,
    debug_shell::DebugShell,
    event_coordinator::EventCoordinator,
    event_interface::EventInterface,
    event_monitor::EventMonitor,
    force_shutdown::ForceShutdown,
    renderer::Renderer,
};

pub type ResourceTypes<RR> = RegAdd![
    <Info as Component>::Storage,
    <Status as Component>::Storage,
    <Camera as Component>::Storage,
    <Renderable as Component>::Storage,
    <UiModel as Component>::Storage,
    <Model as Component>::Storage,
    SceneGraph<UiModel>,
    SceneGraph<Model>,
    EventQueue<EngineEvent>,
    RR
];

pub type UpdateSystemTypes<B, SR> = RegAdd![
    DebugConsole,
    DebugShell,
    ForceShutdown,
    EventMonitor<WorldEvent>,
    EventMonitor<EngineEvent>,
    CameraManager,
    EventCoordinator,
    EventInterface<B>,
    SR
];

pub type RenderSystemTypes<B, SR> = RegAdd![
    Renderer<B>,
    SR
];
