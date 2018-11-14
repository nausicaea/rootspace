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
use ecs::{EventTrait, LoopStage, SystemTrait};
use event::EngineEventTrait;
use failure::Error;
use graphics::{glium::GliumBackend, headless::HeadlessBackend};
use std::time::Duration;

pub enum SystemGroup<E> {
    A(EventCoordinator<Context<E>, E>),
    B(EventMonitor<Context<E>, E>),
    C(GliumEventInterface<Context<E>, E>),
    D(HeadlessEventInterface<Context<E>, E>),
    E(GliumRenderer<Context<E>, E, Renderable<GliumBackend>>),
    F(HeadlessRenderer<Context<E>, E, Renderable<HeadlessBackend>>),
    G(DebugConsole<Context<E>, E>),
    H(DebugShell<Context<E>, E>),
}

impl<E> From<EventCoordinator<Context<E>, E>> for SystemGroup<E> {
    fn from(value: EventCoordinator<Context<E>, E>) -> Self {
        SystemGroup::A(value)
    }
}

impl<E> From<EventMonitor<Context<E>, E>> for SystemGroup<E> {
    fn from(value: EventMonitor<Context<E>, E>) -> Self {
        SystemGroup::B(value)
    }
}

impl<E> From<GliumEventInterface<Context<E>, E>> for SystemGroup<E> {
    fn from(value: GliumEventInterface<Context<E>, E>) -> Self {
        SystemGroup::C(value)
    }
}

impl<E> From<HeadlessEventInterface<Context<E>, E>> for SystemGroup<E> {
    fn from(value: HeadlessEventInterface<Context<E>, E>) -> Self {
        SystemGroup::D(value)
    }
}

impl<E> From<GliumRenderer<Context<E>, E, Renderable<GliumBackend>>> for SystemGroup<E> {
    fn from(value: GliumRenderer<Context<E>, E, Renderable<GliumBackend>>) -> Self {
        SystemGroup::E(value)
    }
}

impl<E> From<HeadlessRenderer<Context<E>, E, Renderable<HeadlessBackend>>> for SystemGroup<E> {
    fn from(value: HeadlessRenderer<Context<E>, E, Renderable<HeadlessBackend>>) -> Self {
        SystemGroup::F(value)
    }
}

impl<E> From<DebugConsole<Context<E>, E>> for SystemGroup<E> {
    fn from(value: DebugConsole<Context<E>, E>) -> Self {
        SystemGroup::G(value)
    }
}

impl<E> From<DebugShell<Context<E>, E>> for SystemGroup<E> {
    fn from(value: DebugShell<Context<E>, E>) -> Self {
        SystemGroup::H(value)
    }
}

impl<E> SystemTrait<Context<E>, E> for SystemGroup<E>
where
    E: EngineEventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        use self::SystemGroup::*;
        match self {
            A(ref s) => s.get_stage_filter(),
            B(ref s) => s.get_stage_filter(),
            C(ref s) => s.get_stage_filter(),
            D(ref s) => s.get_stage_filter(),
            E(ref s) => s.get_stage_filter(),
            F(ref s) => s.get_stage_filter(),
            G(ref s) => s.get_stage_filter(),
            H(ref s) => s.get_stage_filter(),
        }
    }

    fn get_event_filter(&self) -> <E as EventTrait>::EventFlag {
        use self::SystemGroup::*;
        match self {
            A(ref s) => s.get_event_filter(),
            B(ref s) => s.get_event_filter(),
            C(ref s) => s.get_event_filter(),
            D(ref s) => s.get_event_filter(),
            E(ref s) => s.get_event_filter(),
            F(ref s) => s.get_event_filter(),
            G(ref s) => s.get_event_filter(),
            H(ref s) => s.get_event_filter(),
        }
    }

    fn fixed_update(&mut self, ctx: &mut Context<E>, t: &Duration, dt: &Duration) -> Result<(), Error> {
        use self::SystemGroup::*;
        match self {
            A(ref mut s) => s.fixed_update(ctx, t, dt),
            B(ref mut s) => s.fixed_update(ctx, t, dt),
            C(ref mut s) => s.fixed_update(ctx, t, dt),
            D(ref mut s) => s.fixed_update(ctx, t, dt),
            E(ref mut s) => s.fixed_update(ctx, t, dt),
            F(ref mut s) => s.fixed_update(ctx, t, dt),
            G(ref mut s) => s.fixed_update(ctx, t, dt),
            H(ref mut s) => s.fixed_update(ctx, t, dt),
        }
    }

    fn update(&mut self, ctx: &mut Context<E>, t: &Duration, dt: &Duration) -> Result<(), Error> {
        use self::SystemGroup::*;
        match self {
            A(ref mut s) => s.update(ctx, t, dt),
            B(ref mut s) => s.update(ctx, t, dt),
            C(ref mut s) => s.update(ctx, t, dt),
            D(ref mut s) => s.update(ctx, t, dt),
            E(ref mut s) => s.update(ctx, t, dt),
            F(ref mut s) => s.update(ctx, t, dt),
            G(ref mut s) => s.update(ctx, t, dt),
            H(ref mut s) => s.update(ctx, t, dt),
        }
    }

    fn render(&mut self, ctx: &mut Context<E>, t: &Duration, dt: &Duration) -> Result<(), Error> {
        use self::SystemGroup::*;
        match self {
            A(ref mut s) => s.render(ctx, t, dt),
            B(ref mut s) => s.render(ctx, t, dt),
            C(ref mut s) => s.render(ctx, t, dt),
            D(ref mut s) => s.render(ctx, t, dt),
            E(ref mut s) => s.render(ctx, t, dt),
            F(ref mut s) => s.render(ctx, t, dt),
            G(ref mut s) => s.render(ctx, t, dt),
            H(ref mut s) => s.render(ctx, t, dt),
        }
    }

    fn handle_event(&mut self, ctx: &mut Context<E>, event: &E) -> Result<bool, Error> {
        use self::SystemGroup::*;
        match self {
            A(ref mut s) => s.handle_event(ctx, event),
            B(ref mut s) => s.handle_event(ctx, event),
            C(ref mut s) => s.handle_event(ctx, event),
            D(ref mut s) => s.handle_event(ctx, event),
            E(ref mut s) => s.handle_event(ctx, event),
            F(ref mut s) => s.handle_event(ctx, event),
            G(ref mut s) => s.handle_event(ctx, event),
            H(ref mut s) => s.handle_event(ctx, event),
        }
    }
}
