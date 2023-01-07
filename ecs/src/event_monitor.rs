use std::{fmt, time::Duration};

use log::trace;
use serde::{Deserialize, Serialize};

use crate::{
    event_queue::{receiver_id::ReceiverId, EventQueue},
    resources::Resources,
    system::System,
    with_resources::WithResources,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMonitor<E> {
    receiver: ReceiverId<E>,
}

impl<E> WithResources for EventMonitor<E>
where
    E: 'static + Clone + std::fmt::Debug,
{
    fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let receiver = res.borrow_mut::<EventQueue<E>>().subscribe::<Self>();

        Ok(EventMonitor { receiver })
    }
}

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
