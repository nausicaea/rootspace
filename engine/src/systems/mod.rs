pub mod event_interface;
pub mod event_monitor;
pub mod new_renderer;

use self::event_monitor::EventMonitor;
use context::Context;
use event::{Event, EventFlag};

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventMonitor<Event, Context>),
    }
}
