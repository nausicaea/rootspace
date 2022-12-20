use ecs::{EventQueue, RegAdd, WorldEvent};
use crate::{
    events::{window_event::WindowEvent, engine_event::EngineEvent},
    resources::{asset_database::AssetDatabase, graphics::Graphics, statistics::Statistics}, systems::{force_shutdown::ForceShutdown, event_monitor::EventMonitor},
};

pub type Resources<S> = RegAdd![AssetDatabase, Graphics, EventQueue<WindowEvent>, EventQueue<EngineEvent>, Statistics, S];
pub type DynamicSystems<D> = RegAdd![ForceShutdown, EventMonitor<WindowEvent>, EventMonitor<EngineEvent>, EventMonitor<WorldEvent>, D];
