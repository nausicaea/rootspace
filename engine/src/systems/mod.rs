pub mod event_coordinator;
pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

use self::{
    event_coordinator::EventCoordinator,
    event_interface::{GliumEventInterface, HeadlessEventInterface},
    event_monitor::EventMonitor,
    renderer::Renderer,
};
use components::{camera::Camera, model::Model, renderable::Renderable};
use context::Context;
use event::{Event, EventFlag};
use graphics::{glium::{GliumBackend, GliumRenderData}, headless::{HeadlessBackend, HeadlessRenderData}};

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventCoordinator<Context>),
        B(EventMonitor<Context, Event>),
        C(GliumEventInterface<Context, Event>),
        D(HeadlessEventInterface<Context, Event>),
        E(Renderer<Context, Event, Camera, Model, Renderable<GliumBackend>, GliumBackend>),
        F(Renderer<Context, Event, Camera, Model, Renderable<HeadlessBackend>, HeadlessBackend>),
    }
}
