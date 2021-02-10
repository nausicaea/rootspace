use ecs::{EventQueue, ReceiverId, Resources, System, MaybeDefault, WithResources};
use log::trace;
use std::{fmt, time::Duration};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMonitor<E> {
    receiver: ReceiverId<E>,
}

impl<E> WithResources for EventMonitor<E>
where
    E: 'static + Clone + std::fmt::Debug,
{
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<E>>()
            .subscribe::<Self>();

        EventMonitor {
            receiver,
        }
    }
}

impl<E> MaybeDefault for EventMonitor<E> {}

impl<E> System for EventMonitor<E>
where
    E: 'static + Clone + fmt::Debug,
{
    fn name(&self) -> &'static str {
        stringify!(EventMonitor)
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<E>>().receive(&self.receiver);
        for event in events {
            trace!("Received {:?}", event);
        }
    }
}
