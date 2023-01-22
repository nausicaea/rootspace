use ecs::{EventMonitor, EventQueue, RegAdd, WorldEvent};
use rose_tree::Hierarchy;

use crate::{
    components::{
        camera::Camera, info::Info, renderable::Renderable, status::Status, transform::Transform,
        ui_transform::UiTransform,
    },
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics},
    systems::{camera_manager::CameraManager, force_shutdown::ForceShutdown, renderer::Renderer},
};

pub type RRegistry<S> = RegAdd![
    <Camera as ecs::Component>::Storage,
    <Info as ecs::Component>::Storage,
    <Transform as ecs::Component>::Storage,
    <Renderable as ecs::Component>::Storage,
    <Status as ecs::Component>::Storage,
    <UiTransform as ecs::Component>::Storage,
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
