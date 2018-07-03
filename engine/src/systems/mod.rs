pub mod event_coordinator;
pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

use self::event_coordinator::EventCoordinator;
use self::event_interface::{HeadlessEventInterface, GliumEventInterface};
use self::event_monitor::EventMonitor;
use self::renderer::{HeadlessRenderer, GliumRenderer};
use components::model::Model;
use context::Context;
use event::{Event, EventFlag};

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventCoordinator<Context>),
        B(EventMonitor<Context, Event>),
        C(GliumEventInterface<Context, Event>),
        D(HeadlessEventInterface<Context, Event>),
        E(GliumRenderer<Context, Event, Model>),
        F(HeadlessRenderer<Context, Event, Model>),
    }
}
