use ecs::{EventQueue, ReceiverId, Resources, System};
use std::{fmt, time::Duration};
#[cfg(feature = "diagnostics")]
use typename::TypeName;
use log::trace;

pub struct EventMonitor<E> {
    receiver: ReceiverId<E>,
}

#[cfg(not(feature = "diagnostics"))]
impl<E> EventMonitor<E>
where
    E: 'static + Clone,
{
    pub fn new(queue: &mut EventQueue<E>) -> Self {
        EventMonitor {
            receiver: queue.subscribe(),
        }
    }
}

#[cfg(feature = "diagnostics")]
impl<E> EventMonitor<E>
where
    E: 'static + Clone + TypeName,
{
    pub fn new(queue: &mut EventQueue<E>) -> Self {
        EventMonitor {
            receiver: queue.subscribe(),
        }
    }
}

#[cfg(not(feature = "diagnostics"))]
impl<E> System for EventMonitor<E>
where
    E: 'static + Clone + fmt::Debug,
{
    fn name(&self) -> &'static str {
        "EventMonitor"
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<E>>().receive(&self.receiver);
        for event in events {
            trace!("Received {:?}", event);
        }
    }
}

#[cfg(feature = "diagnostics")]
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
            trace!("Received {:?}", event);
        }
    }
}
