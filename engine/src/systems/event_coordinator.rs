use crate::event::EngineEventTrait;
use ecs::{EventHandlerSystem, EventManager, Resources};
use std::marker::PhantomData;
#[cfg(feature = "diagnostics")]
use typename::TypeName;

pub struct EventCoordinator<Evt> {
    _evt: PhantomData<Evt>,
}

impl<Evt> Default for EventCoordinator<Evt> {
    fn default() -> Self {
        EventCoordinator {
            _evt: PhantomData::default(),
        }
    }
}

#[cfg(not(feature = "diagnostics"))]
impl<Evt> EventHandlerSystem<Evt> for EventCoordinator<Evt>
where
    Evt: EngineEventTrait,
{
    fn name(&self) -> &'static str {
        "EventCoordinator"
    }

    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::shutdown() | Evt::hard_shutdown()
    }

    fn run(&mut self, res: &mut Resources, event: &Evt) -> bool {
        if event.matches_filter(Evt::shutdown()) {
            res.get_mut::<EventManager<Evt>>()
                .dispatch_later(Evt::new_hard_shutdown());
            true
        } else if event.matches_filter(Evt::hard_shutdown()) {
            false
        } else {
            true
        }
    }
}

#[cfg(feature = "diagnostics")]
impl<Evt> EventHandlerSystem<Evt> for EventCoordinator<Evt>
where
    Evt: EngineEventTrait + TypeName,
{
    fn name(&self) -> &'static str {
        "EventCoordinator"
    }

    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::shutdown() | Evt::hard_shutdown()
    }

    fn run(&mut self, res: &mut Resources, event: &Evt) -> bool {
        if event.matches_filter(Evt::shutdown()) {
            res.get_mut::<EventManager<Evt>>()
                .dispatch_later(Evt::new_hard_shutdown());
            true
        } else if event.matches_filter(Evt::hard_shutdown()) {
            false
        } else {
            true
        }
    }
}
