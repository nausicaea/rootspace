use ecs::{EventQueue, RegAdd, WorldEvent, EventMonitor};
use rose_tree::Hierarchy;

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics},
    systems::{force_shutdown::ForceShutdown, renderer::Renderer, camera_manager::CameraManager}, components::{info::Info, status::Status, camera::Camera, model::Model, ui_model::UiModel, renderable::Renderable},
};

pub type Resources<S> = RegAdd![
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

pub type DynamicSystems<D> = RegAdd![
    CameraManager,
    ForceShutdown,
    EventMonitor<WindowEvent>,
    EventMonitor<EngineEvent>,
    EventMonitor<WorldEvent>,
    D
];

pub type RenderSystems<R> = RegAdd![
    Renderer,
    R
];
