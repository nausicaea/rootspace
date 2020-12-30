use ecs::{EventQueue, ReceiverId, Resources, System};
use log::trace;
use std::{fmt, time::Duration};

pub struct EventMonitor<E> {
    receiver: ReceiverId<E>,
}

impl<E> EventMonitor<E>
where
    E: 'static + Clone,
{
    pub fn new(queue: &mut EventQueue<E>) -> Self {
        trace!(
            "{} subscribing to {}",
            std::any::type_name::<Self>(),
            std::any::type_name::<EventQueue<E>>()
        );
        EventMonitor {
            receiver: queue.subscribe(),
        }
    }
}

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
