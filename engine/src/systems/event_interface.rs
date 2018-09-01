use ecs::{EventManagerTrait, EventTrait, LoopStage, SystemTrait};
use failure::Error;
use graphics::{
    glium::{GliumEvent as GE, GliumEventsLoop as GEL},
    headless::{HeadlessEvent as HE, HeadlessEventsLoop as HEL},
    EventsLoopTrait,
};
use std::{convert::TryInto, marker::PhantomData, time::Duration};

pub type HeadlessEventInterface<Ctx, Evt> = EventInterface<Ctx, Evt, HEL, HE>;
pub type GliumEventInterface<Ctx, Evt> = EventInterface<Ctx, Evt, GEL, GE>;

pub struct EventInterface<Ctx, Evt, L, I> {
    pub events_loop: L,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
    _i: PhantomData<I>,
}

impl<Ctx, Evt, L, I> EventInterface<Ctx, Evt, L, I> {
    pub fn new(events_loop: L) -> Self {
        EventInterface {
            events_loop,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
            _i: PhantomData::default(),
        }
    }
}

impl<Ctx, Evt, L, I> Default for EventInterface<Ctx, Evt, L, I>
where
    L: Default,
{
    fn default() -> Self {
        EventInterface::new(Default::default())
    }
}

impl<Ctx, Evt, L, I> SystemTrait<Ctx, Evt> for EventInterface<Ctx, Evt, L, I>
where
    Ctx: EventManagerTrait<Evt>,
    Evt: EventTrait,
    L: EventsLoopTrait<Evt, I>,
    I: TryInto<Evt>,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::UPDATE
    }

    fn update(&mut self, ctx: &mut Ctx, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        self.events_loop.poll(|input_event| {
            if let Ok(event) = input_event.try_into() {
                ctx.dispatch_later(event);
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use components::model::Model;
    use event::Event;
    use mock::MockCtx;

    #[test]
    fn new_headless() {
        let _: HeadlessEventInterface<MockCtx<Event, Model>, Event> = EventInterface::default();
    }

    #[test]
    fn get_stage_filter_headless() {
        let e: HeadlessEventInterface<MockCtx<Event, Model>, Event> = EventInterface::default();

        assert_eq!(e.get_stage_filter(), LoopStage::UPDATE);
    }

    #[test]
    fn update_headless() {
        let mut e: HeadlessEventInterface<MockCtx<Event, Model>, Event> = EventInterface::default();
        let mut c = MockCtx::default();

        assert_ok!(e.update(&mut c, &Default::default(), &Default::default()));
        assert_eq!(c.dispatch_later_calls, 0);
        assert!(c.events.is_empty());
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic)]
    fn new_glium() {
        let _: GliumEventInterface<MockCtx<Event, Model>, Event> = EventInterface::default();
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic)]
    fn get_stage_filter_glium() {
        let e: GliumEventInterface<MockCtx<Event, Model>, Event> = EventInterface::default();

        assert_eq!(e.get_stage_filter(), LoopStage::UPDATE);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic)]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Events can only be polled from the main thread on macOS")
    )]
    fn update_glium() {
        let mut e: GliumEventInterface<MockCtx<Event, Model>, Event> = EventInterface::default();
        let mut c = MockCtx::default();

        assert_ok!(e.update(&mut c, &Default::default(), &Default::default()));
    }
}
