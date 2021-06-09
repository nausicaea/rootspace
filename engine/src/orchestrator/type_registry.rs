use ecs::{Component, EventQueue, RegAdd, WorldEvent};

use crate::{
    components::{Camera, Info, Model, Renderable, Status, UiModel},
    event::EngineEvent,
    resources::{AssetDatabase, GraphicsBackend, SceneGraph, Settings, Statistics},
    systems::{
        CameraManager, DebugConsole, DebugShell, EventCoordinator, EventInterface, EventMonitor, ForceShutdown,
        Renderer,
    },
};

pub type ResourceTypes<B, RR> = RegAdd![
    <Info as Component>::Storage,
    <Status as Component>::Storage,
    <Camera as Component>::Storage,
    <Renderable as Component>::Storage,
    <UiModel as Component>::Storage,
    <Model as Component>::Storage,
    SceneGraph<UiModel>,
    SceneGraph<Model>,
    EventQueue<EngineEvent>,
    GraphicsBackend<B>,
    AssetDatabase,
    Statistics,
    Settings,
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

pub type RenderSystemTypes<B, SR> = RegAdd![Renderer<B>, SR];
