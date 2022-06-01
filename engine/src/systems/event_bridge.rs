use std::{convert::TryInto, marker::PhantomData, time::Duration};

use ecs::{EventQueue, Resources, SerializationName, System};
use serde::{Deserialize, Serialize};

use crate::{event::EngineEvent, graphics::BackendTrait, resources::GraphicsBackend};

#[derive(Serialize, Deserialize)]
#[serde(rename(serialize = "EventBridge", deserialize = "EventBridge"))]
pub struct EventBridge<B> {
    #[serde(skip)]
    _b: PhantomData<B>,
}

impl<B> Default for EventBridge<B> {
    fn default() -> Self {
        EventBridge {
            _b: PhantomData::default(),
        }
    }
}

impl<B> SerializationName for EventBridge<B>
where
    B: BackendTrait,
{
    fn name() -> String {
        String::from("EventBridge")
    }
}

impl<B> std::fmt::Debug for EventBridge<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "EventBridge<{}>(PhantomData)", std::any::type_name::<B>())
    }
}

impl<B> System for EventBridge<B>
where
    B: BackendTrait,
{
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let mut events: Vec<EngineEvent> = Vec::default();

        res.borrow_mut::<GraphicsBackend<B>>()
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
    fn default_headless() {
        let _: EventBridge<HeadlessBackend> = EventBridge::default();
    }
}
