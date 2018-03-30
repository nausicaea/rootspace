use std::fmt::Debug;
use failure::Error;

pub trait EventTrait: Clone + Debug {
    type EventFlag: Default + Clone + Copy;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool;
}

pub trait EventManagerTrait<E: EventTrait>: Default {
    fn dispatch_later(&mut self, event: E);
    fn handle_events<F>(&mut self, handler: F) -> Result<bool, Error>
        where F: FnMut(&mut Self, &E) -> Result<bool, Error>;
}
