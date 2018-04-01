use std::time::Duration;
use failure::Error;
use ecs::system::SystemTrait;
use ecs::loop_stage::LoopStage;
use engine::event_monitor::EventMonitor;
use event::{Event, EventFlag};
use context::Context;

impl_system_group! {
    pub enum SystemGroup<Context<Event>, Event, EventFlag> {
        A(EventMonitor<Context<Event>, Event>),
    }
}
