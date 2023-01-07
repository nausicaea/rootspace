use ecs::{EventMonitor, EventQueue, RegAdd, WorldEvent};
use rose_tree::Hierarchy;

use crate::{
    components::{camera::Camera, info::Info, model::Model, renderable::Renderable, status::Status, ui_model::UiModel},
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics},
    systems::{camera_manager::CameraManager, force_shutdown::ForceShutdown, renderer::Renderer},
};

pub type RRegistry<S> = RegAdd![
    <Camera as ecs::Component>::Storage,
    <Info as ecs::Component>::Storage,
    <Model as ecs::Component>::Storage,
    <Renderable as ecs::Component>::Storage,
    <Status as ecs::Component>::Storage,
    <UiModel as ecs::Component>::Storage,
    AssetDatabase,
    EventQueue<WindowEvent>,
    EventQueue<EngineEvent>,
    Graphics,
    Hierarchy<ecs::Index>,
    Statistics,
    S
];

pub type USRegistry<D> = RegAdd![
    CameraManager,
    ForceShutdown,
    EventMonitor<WindowEvent>,
    EventMonitor<EngineEvent>,
    EventMonitor<WorldEvent>,
    D
];

pub type RSRegistry<R> = RegAdd![Renderer, R];
