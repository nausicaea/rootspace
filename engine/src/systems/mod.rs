pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

use self::event_interface::EventInterface;
use self::event_monitor::EventMonitor;
use self::renderer::Renderer;
use components::model::Model;
use context::Context;
use mock::MockDisplay;
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
        C(Renderer<Event, Context, Display, Model>),
        D(EventInterface<Event, Context, ()>),
        E(Renderer<Event, Context, MockDisplay, Model>),
    }
}
