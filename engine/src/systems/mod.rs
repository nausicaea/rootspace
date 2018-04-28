pub mod event_monitor;
pub mod event_interface;
pub mod renderer;

use std::time::Duration;
use failure::Error;
use winit::EventsLoop;
use ecs::system::SystemTrait;
use ecs::loop_stage::LoopStage;
use event::{Event, EventFlag};
use context::Context;
use self::event_monitor::EventMonitor;
use self::event_interface::EventInterface;

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventMonitor<Event, Context>),
        B(EventInterface<Event, Context, EventsLoop>),
    }
}
