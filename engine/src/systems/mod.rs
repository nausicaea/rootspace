pub mod event_monitor;
pub mod event_interface;
pub mod open_gl_renderer;

use std::time::Duration;
use failure::Error;
use winit::EventsLoop;
use glium::Display;
use ecs::system::SystemTrait;
use ecs::loop_stage::LoopStage;
use event::{Event, EventFlag};
use context::Context;
use self::event_monitor::EventMonitor;
use self::event_interface::EventInterface;
use self::open_gl_renderer::OpenGlRenderer;

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventMonitor<Event, Context>),
        B(EventInterface<Event, Context, EventsLoop>),
        C(OpenGlRenderer<Event, Context, Display>),
    }
}
