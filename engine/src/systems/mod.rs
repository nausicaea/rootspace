pub mod event_interface;
pub mod event_monitor;
pub mod open_gl_renderer;

use self::event_interface::EventInterface;
use self::event_monitor::EventMonitor;
use self::open_gl_renderer::OpenGlRenderer;
use components::model::Model;
use context::Context;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use event::{Event, EventFlag};
use failure::Error;
use glium::glutin::EventsLoop;
use glium::Display;
use std::time::Duration;

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventMonitor<Event, Context>),
        B(EventInterface<Event, Context, EventsLoop>),
        C(OpenGlRenderer<Event, Context, Display, Model>),
    }
}
