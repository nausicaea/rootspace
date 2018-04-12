use std::time::Duration;
use failure::Error;
use winit::EventsLoop;
use ecs::system::SystemTrait;
use ecs::loop_stage::LoopStage;
use event::{Event, EventFlag};
use context::Context;
use event_monitor::EventMonitor;
use event_interface::EventInterface;

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventMonitor<Event, Context>),
        B(EventInterface<Event, Context, EventsLoop>),
    }
}
