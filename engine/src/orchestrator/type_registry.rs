use ecs::{Component, EventQueue, Index, RegAdd, WorldEvent};
use rose_tree::Hierarchy;

use crate::{
    components::{Camera, Info, Model, Renderable, Status, UiModel},
    event::EngineEvent,
    resources::{AssetDatabase, GraphicsBackend, Settings, Statistics},
    systems::{
        CameraManager, DebugConsole, DebugInteractions, DebugShell, EventCoordinator, EventInterface, EventMonitor,
        ForceShutdown, Renderer,
    },
};

pub type ResourceTypes<B, RR> = RegAdd![
    <Info as Component>::Storage,
    <Status as Component>::Storage,
    <Camera as Component>::Storage,
    <Renderable as Component>::Storage,
    <UiModel as Component>::Storage,
    <Model as Component>::Storage,
    Hierarchy<Index>,
    EventQueue<EngineEvent>,
    GraphicsBackend<B>,
    AssetDatabase,
    Statistics,
    Settings,
    RR
];

pub type UpdateSystemTypes<B, SR> = RegAdd![
    DebugInteractions,
    DebugConsole,
    DebugShell<B>,
    ForceShutdown,
    EventMonitor<WorldEvent>,
    EventMonitor<EngineEvent>,
    CameraManager,
    EventCoordinator,
    EventInterface<B>,
    SR
];

pub type RenderSystemTypes<B, SR> = RegAdd![Renderer<B>, SR];
