use ecs::{EventQueue, ReceiverId, Resources, System};
use std::fmt;
use std::time::Duration;
#[cfg(feature = "diagnostics")]
use typename::TypeName;

pub struct EventMonitor<E> {
    receiver: ReceiverId<E>,
}

#[cfg(not(feature = "diagnostics"))]
impl<E> EventMonitor<E>
where
    E: 'static + Clone,
{
    pub fn new(res: &mut Resources) -> Self {
        let events = res.get_mut::<EventQueue<E>>();
        let receiver = events.subscribe();

        EventMonitor { receiver }
    }
}

#[cfg(feature = "diagnostics")]
impl<E> EventMonitor<E>
where
    E: 'static + Clone + TypeName,
{
    pub fn new(res: &mut Resources) -> Self {
        let events = res.get_mut::<EventQueue<E>>();
        let receiver = events.subscribe();

        EventMonitor { receiver }
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
