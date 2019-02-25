use crate::resources::Resource;
use std::fmt;
use std::collections::VecDeque;
use std::ops::{BitAnd, BitOr, BitXor};

pub trait EventTrait: Clone + fmt::Debug + 'static {
    type EventFlag: Default + Clone + Copy + PartialEq + BitAnd<Output = Self::EventFlag> + BitOr<Output = Self::EventFlag> + BitXor<Output = Self::EventFlag>;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool;
}

pub struct EventManager<E>(VecDeque<E>);

impl<E> EventManager<E> {
    pub fn dispatch_later(&mut self, event: E) {
        self.0.push_back(event)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn flush(&mut self) -> Vec<E> {
        self.0.drain(..).collect()
    }
}

impl<E> Default for EventManager<E> {
    fn default() -> Self {
        EventManager(VecDeque::default())
    }
}

impl<E> fmt::Debug for EventManager<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventManager {{ ... }}")
    }
}

impl<E> Resource for EventManager<E> where E: 'static {}
