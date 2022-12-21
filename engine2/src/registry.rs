use ecs::{EventQueue, RegAdd, WorldEvent};

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics},
    systems::{event_monitor::EventMonitor, force_shutdown::ForceShutdown},
};

pub type Resources<S> = RegAdd![
    AssetDatabase,
    Graphics,
    EventQueue<WindowEvent>,
    EventQueue<EngineEvent>,
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
