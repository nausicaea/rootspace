use std::{fmt, time::Duration};

use ecs::{event_queue::receiver_id::ReceiverId, EventQueue, Resources, SerializationName, System, WithResources};
use log::trace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMonitor<E> {
    receiver: ReceiverId<E>,
}

impl<E> WithResources for EventMonitor<E>
where
    E: 'static + Clone + std::fmt::Debug,
{
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<E>>().subscribe::<Self>();

        EventMonitor { receiver }
    }
}

impl<E> SerializationName for EventMonitor<E> {}

impl<E> System for EventMonitor<E>
where
    E: 'static + Clone + fmt::Debug,
{
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<E>>().receive(&self.receiver);
        for event in events {
            trace!("Received {:?}", event);
        }
    }
}
