use crate::{event::EngineEvent, graphics::BackendTrait, resources::BackendResource};
use ecs::{EventQueue, Resources, System};
use std::{convert::TryInto, marker::PhantomData, time::Duration};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct EventInterface<B>(PhantomData<B>);

impl<B> Default for EventInterface<B> {
    fn default() -> Self {
        EventInterface(PhantomData::default())
    }
}

impl<B> std::fmt::Debug for EventInterface<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "EventInterface<{}>(PhantomData)", std::any::type_name::<B>())
    }
}

impl<B> System for EventInterface<B>
where
    B: BackendTrait
{
    fn name(&self) -> &'static str {
        stringify!(EventInterface)
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let mut events: Vec<EngineEvent> = Vec::default();

        res.borrow_mut::<BackendResource<B>>()
            .poll_events(|input_event: B::Event| {
                if let Ok(event) = input_event.try_into() {
                    events.push(event);
                }
            });

        let mut queue = res.borrow_mut::<EventQueue<EngineEvent>>();
        events.into_iter().for_each(|e| queue.send(e));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::headless::HeadlessBackend;

    #[test]
    fn new_headless() {
        let _: EventInterface<HeadlessBackend> = EventInterface::default();
    }
}
