pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

use self::event_interface::{HeadlessEventInterface, GliumEventInterface};
use self::event_monitor::EventMonitor;
use self::renderer::{HeadlessRenderer, GliumRenderer};
use components::model::Model;
use context::Context;
use event::{Event, EventFlag};

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventMonitor<Event, Context>),
        B(GliumRenderer<Context, Event, Model>),
        C(HeadlessRenderer<Context, Event, Model>),
        D(GliumEventInterface<Context, Event>),
        E(HeadlessEventInterface<Context, Event>),
    }
}
