use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};

use super::{
    event_queue::{EventQueue, receiver_id::ReceiverId},
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
    E: 'static + Clone + fmt::Debug + Send + Sync,
{
    #[tracing::instrument(skip_all)]
    fn with_res(res: &Resources) -> anyhow::Result<Self> {
        let receiver = res.write::<EventQueue<E>>().subscribe::<Self>();

        Ok(EventMonitor { receiver })
    }
}

impl<E> System for EventMonitor<E>
where
    E: 'static + Clone + fmt::Debug + Send + Sync,
{
    #[tracing::instrument(skip_all)]
    fn run(&mut self, res: &Resources, _t: Duration, _dt: Duration) {
        res.write::<EventQueue<E>>()
            .receive_cb(&self.receiver, |e| tracing::trace!("Received {:?}", e));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Reg,
        registry::{End, SystemRegistry},
        world::World,
    };

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
        let _w =
            World::with_dependencies::<Reg![EventQueue<usize>], Reg![], Reg![EventMonitor<usize>], (), Reg![], _>(&())
                .unwrap();
    }
}
