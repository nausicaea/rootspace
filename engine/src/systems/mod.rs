pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

use self::event_interface::EventInterface;
use self::event_monitor::EventMonitor;
use self::renderer::Renderer;
use components::model::Model;
use components::renderable::{GliumRenderData, HeadlessRenderData, Renderable};
use context::Context;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use event::{Event, EventFlag};
use failure::Error;
use glium::glutin::EventsLoop;
use glium::Display;
use graphics::headless::{HeadlessDisplay, HeadlessEventsLoop};
use std::time::Duration;

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventMonitor<Event, Context>),
        B(EventInterface<Event, Context, EventsLoop>),
        C(Renderer<Event, Context, Display, Model, Renderable<GliumRenderData>>),
        D(EventInterface<Event, Context, HeadlessEventsLoop>),
        E(Renderer<Event, Context, HeadlessDisplay, Model, Renderable<HeadlessRenderData>>),
    }
}
