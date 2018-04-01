use std::collections::VecDeque;
use failure::Error;
use ecs::event::{EventTrait, EventManagerTrait};

pub struct Context<E>
where
    E: EventTrait,
{
    events: VecDeque<E>,
}

impl<E> Default for Context<E>
where
    E: EventTrait,
{
    fn default() -> Self {
        Context {
            events: Default::default(),
        }
    }
}

impl<E> EventManagerTrait<E> for Context<E>
where
    E: EventTrait,
{
    fn dispatch_later(&mut self, event: E) {
        self.events.push_back(event)
    }
    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
    where
        F: FnMut(&mut Self, &E) -> Result<bool, Error>,
    {
        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        for event in tmp {
            handler(self, &event)?;
        }

        Ok(true)
    }
}
