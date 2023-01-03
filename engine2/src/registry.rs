use ecs::{EventQueue, RegAdd, WorldEvent, EventMonitor};
use rose_tree::Hierarchy;

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics},
    systems::{force_shutdown::ForceShutdown, renderer::Renderer},
};

pub type Resources<S> = RegAdd![
    AssetDatabase,
    EventQueue<EngineEvent>,
    EventQueue<WindowEvent>,
    Graphics,
    Hierarchy<ecs::Index>,
    Statistics,
    S
];

pub type DynamicSystems<D> = RegAdd![
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
