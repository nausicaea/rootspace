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
        res.borrow_mut::<EventQueue<E>>()
            .receive_cb(&self.receiver, |e| trace!("Received {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{End, Reg, SystemRegistry, World};

    #[test]
    fn event_monitor_reg_macro() {
        type _SR = Reg![EventMonitor<u32>];
    }

    #[test]
    fn event_monitor_system_registry() {
        let res = Resources::with_dependencies::<Reg![EventQueue<usize>], _>(&()).unwrap();
        let _rr = SystemRegistry::push(End, EventMonitor::<usize>::with_res(&res).unwrap());
    }

    #[test]
    fn event_monitor_world() {
        let _w = World::with_dependencies::<Reg![EventQueue<usize>], Reg![], Reg![EventMonitor<usize>], Reg![], _>(&())
            .unwrap();
    }
}
