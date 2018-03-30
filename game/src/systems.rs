use std::time::Duration;
use failure::Error;
use ecs::system::SystemTrait;
use ecs::loop_stage::LoopStage;
use ecs::database::Database;
use engine::event_monitor::EventMonitor;
use event::{EventManager, Event, EventFlag};
use auxiliary::Auxiliary;

impl_system_group! {
    pub enum SystemGroup<EventManager, Auxiliary, Database, Event, EventFlag> {
        A(EventMonitor<EventManager, Auxiliary, Database, Event>),
    }
}
