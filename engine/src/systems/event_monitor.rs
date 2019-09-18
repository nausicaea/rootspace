use ecs::{EventQueue, ReceiverId, Resources, System};
use log::{debug, trace};
use std::{fmt, time::Duration};
use typename::TypeName;

#[derive(TypeName)]
pub struct EventMonitor<E> {
    receiver: ReceiverId<E>,
}

impl<E> EventMonitor<E>
where
    E: 'static + Clone + TypeName,
{
    pub fn new(queue: &mut EventQueue<E>) -> Self {
        trace!("{} subscribing to {}", Self::type_name(), queue.type_name_of());
        EventMonitor {
            receiver: queue.subscribe(),
        }
    }
}

impl<E> System for EventMonitor<E>
where
    E: 'static + Clone + fmt::Debug + TypeName,
{
    fn name(&self) -> &'static str {
        "EventMonitor"
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<E>>().receive(&self.receiver);
        for event in events {
            debug!("Received {:?}", event);
        }
    }
}
