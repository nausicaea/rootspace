pub mod event_coordinator;
pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

use self::event_coordinator::EventCoordinator;
use self::event_interface::{GliumEventInterface, HeadlessEventInterface};
use self::event_monitor::EventMonitor;
use self::renderer::{GliumRenderer, HeadlessRenderer};
use components::camera::Camera;
use components::model::Model;
use context::Context;
use event::{Event, EventFlag};

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventCoordinator<Context>),
        B(EventMonitor<Context, Event>),
        C(GliumEventInterface<Context, Event>),
        D(HeadlessEventInterface<Context, Event>),
        E(GliumRenderer<Context, Event, Camera, Model>),
        F(HeadlessRenderer<Context, Event, Camera, Model>),
    }
}
