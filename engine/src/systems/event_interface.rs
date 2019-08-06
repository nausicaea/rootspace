use crate::{
    event::EngineEventTrait,
    graphics::{BackendTrait, EventsLoopTrait},
};
use ecs::{EventQueue, Resources, System};
use std::{convert::TryFrom, marker::PhantomData, time::Duration};
#[cfg(feature = "diagnostics")]
use typename::TypeName;

pub struct EventInterface<Evt, B: BackendTrait> {
    pub events_loop: B::EventsLoop,
    _evt: PhantomData<Evt>,
    _b: PhantomData<B>,
}

impl<Evt, B> Default for EventInterface<Evt, B>
where
    B: BackendTrait,
{
    fn default() -> Self {
        EventInterface {
            events_loop: B::EventsLoop::default(),
            _evt: PhantomData::default(),
            _b: PhantomData::default(),
        }
    }
}

#[cfg(not(feature = "diagnostics"))]
impl<Evt, B> System for EventInterface<Evt, B>
where
    Evt: EngineEventTrait + TryFrom<B::Event>,
    B: BackendTrait,
{
    fn name(&self) -> &'static str {
        "EventInterface"
    }

    fn run(&mut self, res: &mut Resources, _t: &Duration, _dt: &Duration) {
        self.events_loop.poll(|input_event: B::Event| {
            if let Ok(event) = TryFrom::try_from(input_event) {
                res.get_mut::<EventQueue<Evt>>().dispatch_later(event);
            }
        });
    }
}

#[cfg(feature = "diagnostics")]
impl<Evt, B> System for EventInterface<Evt, B>
where
    Evt: EngineEventTrait + TryFrom<B::Event> + TypeName,
    B: BackendTrait,
{
    fn name(&self) -> &'static str {
        "EventInterface"
    }

    fn run(&mut self, res: &mut Resources, _t: &Duration, _dt: &Duration) {
        self.events_loop.poll(|input_event: B::Event| {
            if let Ok(event) = TryFrom::try_from(input_event) {
                res.get_mut::<EventQueue<Evt>>().dispatch_later(event);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graphics::headless::HeadlessBackend, mock::MockEvt};

    #[test]
    fn new_headless() {
        let _: EventInterface<MockEvt, HeadlessBackend> = EventInterface::default();
    }
}
