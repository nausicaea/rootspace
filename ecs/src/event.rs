use failure::Error;
use std::fmt::Debug;
use std::ops;

pub trait EventTrait: Clone + Debug {
    type EventFlag: Default + Clone + Copy + PartialEq + ops::BitAnd<Output = Self::EventFlag> + ops::BitOr<Output = Self::EventFlag> + ops::BitXor<Output = Self::EventFlag>;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool;
}

pub trait EventManagerTrait<E: EventTrait> {
    fn dispatch_later(&mut self, event: E);
    fn handle_events<F>(&mut self, handler: F) -> Result<bool, Error>
    where
        F: FnMut(&mut Self, &E) -> Result<bool, Error>;
}
