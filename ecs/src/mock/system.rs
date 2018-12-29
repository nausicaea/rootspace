use crate::event::EventTrait;
use std::{marker::PhantomData, time::Duration, sync::RwLock};
use crate::system::{System, EventHandlerSystem};

#[derive(Debug, Default)]
pub struct MockFixedUpdateSys<C> {
    calls: usize,
    _ctx: PhantomData<C>,
}

impl<C> System<C> for MockFixedUpdateSys<C> {
    fn run(&mut self, _ctx: &mut C, _t: &Duration, _dt: &Duration) {
        self.calls += 1;
    }
}

#[derive(Debug, Default)]
pub struct MockUpdateSys<C> {
    calls: usize,
    _ctx: PhantomData<C>,
}

impl<C> System<C> for MockUpdateSys<C> {
    fn run(&mut self, _ctx: &mut C, _t: &Duration, _dt: &Duration) {
        self.calls += 1;
    }
}

#[derive(Debug, Default)]
pub struct MockRenderSys<C> {
    calls: usize,
    _ctx: PhantomData<C>,
}

impl<C> System<C> for MockRenderSys<C> {
    fn run(&mut self, _ctx: &mut C, _t: &Duration, _dt: &Duration) {
        self.calls += 1;
    }
}

#[derive(Debug)]
pub struct MockEventHandlerSys<C, E> {
    calls: usize,
    filter_calls: RwLock<usize>,
    return_code: bool,
    _ctx: PhantomData<C>,
    _evt: PhantomData<E>,
}

impl<C, E> MockEventHandlerSys<C, E> {
    pub fn with_return_code(rc: bool) -> Self {
        MockEventHandlerSys {
            return_code: rc,
            .. Default::default()
        }
    }
}

impl<C, E> Default for MockEventHandlerSys<C, E> {
    fn default() -> Self {
        MockEventHandlerSys {
            calls: 0,
            filter_calls: RwLock::new(0),
            return_code: true,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        }
    }
}

impl<C, E> EventHandlerSystem<C, E> for MockEventHandlerSys<C, E>
where
    E: EventTrait,
{
    fn get_event_filter(&self) -> E::EventFlag {
        self.filter_calls
            .write()
            .map(|mut c| *c += 1)
            .expect("Could not increment the get_event_filter call counter");

        Default::default()
    }

    fn run(&mut self, _ctx: &mut C, _e: &E) -> bool {
        self.calls += 1;
        self.return_code
    }
}
