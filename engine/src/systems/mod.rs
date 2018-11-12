pub mod debug_console;
pub mod debug_shell;
pub mod event_coordinator;
pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

pub use self::{
    debug_console::DebugConsole,
    debug_shell::DebugShell,
    event_coordinator::EventCoordinator,
    event_interface::{GliumEventInterface, HeadlessEventInterface},
    event_monitor::EventMonitor,
    renderer::{GliumRenderer, HeadlessRenderer},
};
use components::renderable::Renderable;
use context::Context;
use event::{Event, EventFlag};
use graphics::{glium::GliumBackend, headless::HeadlessBackend};

impl_system_group! {
    pub enum SystemGroup<Context, Event, EventFlag> {
        A(EventCoordinator<Context>),
        B(EventMonitor<Context, Event>),
        C(GliumEventInterface<Context, Event>),
        D(HeadlessEventInterface<Context, Event>),
        E(GliumRenderer<Context, Event, Renderable<GliumBackend>>),
        F(HeadlessRenderer<Context, Event, Renderable<HeadlessBackend>>),
        G(DebugConsole<Context>),
        H(DebugShell<Context>),
    }
}
