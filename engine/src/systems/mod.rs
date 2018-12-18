pub mod debug_console;
pub mod debug_shell;
pub mod event_coordinator;
pub mod event_interface;
pub mod event_monitor;
pub mod renderer;

pub use self::{
    debug_console::DebugConsole, debug_shell::DebugShell, event_coordinator::EventCoordinator,
    event_interface::EventInterface, event_monitor::EventMonitor, renderer::Renderer,
};
use crate::context::SceneGraphTrait;
use ecs::{DatabaseTrait, EventManagerTrait, EventTrait, LoopStage, SystemTrait};
use crate::event::EngineEventTrait;
use failure::Error;
use crate::graphics::{
    glium::{GliumBackend, GliumEventsLoop},
    headless::{HeadlessBackend, HeadlessEventsLoop},
};
use std::time::Duration;

pub enum SystemGroup<C, E> {
    A(EventCoordinator<C, E>),
    B(EventMonitor<C, E>),
    C(EventInterface<C, E, GliumEventsLoop>),
    D(EventInterface<C, E, HeadlessEventsLoop>),
    E(Renderer<C, E, GliumBackend>),
    F(Renderer<C, E, HeadlessBackend>),
    G(DebugConsole<C, E>),
    H(DebugShell<C, E>),
}

impl<C, E> From<EventCoordinator<C, E>> for SystemGroup<C, E> {
    fn from(value: EventCoordinator<C, E>) -> Self {
        SystemGroup::A(value)
    }
}

impl<C, E> From<EventMonitor<C, E>> for SystemGroup<C, E> {
    fn from(value: EventMonitor<C, E>) -> Self {
        SystemGroup::B(value)
    }
}

impl<C, E> From<EventInterface<C, E, GliumEventsLoop>> for SystemGroup<C, E> {
    fn from(value: EventInterface<C, E, GliumEventsLoop>) -> Self {
        SystemGroup::C(value)
    }
}

impl<C, E> From<EventInterface<C, E, HeadlessEventsLoop>> for SystemGroup<C, E> {
    fn from(value: EventInterface<C, E, HeadlessEventsLoop>) -> Self {
        SystemGroup::D(value)
    }
}

impl<C, E> From<Renderer<C, E, GliumBackend>> for SystemGroup<C, E> {
    fn from(value: Renderer<C, E, GliumBackend>) -> Self {
        SystemGroup::E(value)
    }
}

impl<C, E> From<Renderer<C, E, HeadlessBackend>> for SystemGroup<C, E> {
    fn from(value: Renderer<C, E, HeadlessBackend>) -> Self {
        SystemGroup::F(value)
    }
}

impl<C, E> From<DebugConsole<C, E>> for SystemGroup<C, E> {
    fn from(value: DebugConsole<C, E>) -> Self {
        SystemGroup::G(value)
    }
}

impl<C, E> From<DebugShell<C, E>> for SystemGroup<C, E> {
    fn from(value: DebugShell<C, E>) -> Self {
        SystemGroup::H(value)
    }
}

impl<C, E> SystemTrait<C, E> for SystemGroup<C, E>
where
    E: EngineEventTrait,
    C: DatabaseTrait + EventManagerTrait<E> + SceneGraphTrait + 'static,
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

    fn fixed_update(&mut self, ctx: &mut C, t: &Duration, dt: &Duration) -> Result<(), Error> {
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

    fn update(&mut self, ctx: &mut C, t: &Duration, dt: &Duration) -> Result<(), Error> {
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

    fn render(&mut self, ctx: &mut C, t: &Duration, dt: &Duration) -> Result<(), Error> {
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

    fn handle_event(&mut self, ctx: &mut C, event: &E) -> Result<bool, Error> {
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
