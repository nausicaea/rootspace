//! Provides facilities to define and manage events.

use crate::resources::Resource;
use std::{
    collections::{VecDeque, HashMap},
    fmt,
    ops::{BitAnd, BitOr, BitXor},
    marker::PhantomData,
};

/// Events sent around within the `World` need to implement this trait such that individual
/// `EventHandlerSystem`s may filter for particular events.
pub trait EventTrait: Clone + 'static {
    /// Defines the event filter type.
    type EventFlag: Default
        + Clone
        + Copy
        + PartialEq
        + BitAnd<Output = Self::EventFlag>
        + BitOr<Output = Self::EventFlag>
        + BitXor<Output = Self::EventFlag>;

    /// Given an event filter, returns `true` if the current event matches that filter, `false`
    /// otherwise.
    ///
    /// # Arguments
    ///
    /// * `flag` - The filter (or bitflag) with which to evaluate the current event.
    fn matches_filter(&self, flag: Self::EventFlag) -> bool;
}

/// A handle that allows a receiver to receive events from the related event queue.
#[derive(Debug, Clone)]
pub struct ReceiverId<E> {
    id: usize,
    _e: PhantomData<E>,
}

impl<E> ReceiverId<E> {
    fn new(id: usize) -> Self {
        ReceiverId {
            id,
            _e: PhantomData::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct ReceiverState<E> {
    events_read: usize,
    events_received: usize,
    _e: PhantomData<E>,
}

impl<E> Default for ReceiverState<E> {
    fn default() -> Self {
        ReceiverState {
            events_read: 0,
            events_received: 0,
            _e: PhantomData::default(),
        }
    }
}

/// An `EventQueue` contains a queue of events and provides rudimentary facilities of retrieving
/// those events.
#[cfg_attr(feature = "diagnostics", derive(TypeName))]
pub struct EventQueue<E>{
    events: VecDeque<E>,
    max_id: usize,
    receivers: HashMap<usize, ReceiverState<E>>,
    free_ids: Vec<usize>,
}

impl<E> EventQueue<E>
where
    E: Clone,
{
    /// Subscribe to this event queue.
    pub fn subscribe(&mut self) -> ReceiverId<E> {
        let id = if let Some(id) = self.free_ids.pop() {
            id
        } else {
            let tmp = self.max_id;
            self.max_id += 1;
            tmp
        };

        self.receivers.insert(id, ReceiverState::default());

        ReceiverId::new(id)
    }

    /// Unsubscribe from this event queue.
    pub fn unsubscribe(&mut self, id: ReceiverId<E>) {
        self.receivers.remove(&id.id);
        self.free_ids.push(id.id);
    }

    /// Send an event into the queue.
    pub fn send(&mut self, event: E) {
        if !self.receivers.is_empty() {
            self.events.push_front(event);
            self.receivers.values_mut()
                .for_each(|s| s.events_received += 1);
        }
    }

    /// Receive all unread events from the queue.
    pub fn receive(&mut self, id: &ReceiverId<E>) -> Vec<E> {
        let events = &self.events;
        let evs: Vec<E> = self.receivers.get_mut(&id.id)
            .filter(|s| s.events_read < s.events_received)
            .map(|s| {
                let unread = s.events_received - s.events_read;
                s.events_read += unread;
                events.iter().take(unread).cloned().collect()
            })
            .unwrap_or_default();

        let unread = self.receivers
            .values()
            .map(|s| s.events_received - s.events_read)
            .max()
            .unwrap_or_default();
        self.events.truncate(unread);

        evs
    }

    /// Return the number of queued events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Return the number of subscribers to this queue.
    pub fn subscribers(&self) -> usize {
        self.receivers.len()
    }

    /// Dispatches an event to the queue.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to add to the queue.
    pub fn dispatch_later(&mut self, event: E) {
        self.events.push_back(event);
    }

    /// Empties the queue and returns all queued events in FIFO (first-in, first-out) order.
    pub fn flush(&mut self) -> Vec<E> {
        self.events.drain(..).collect()
    }
}

impl<E> Default for EventQueue<E> {
    fn default() -> Self {
        EventQueue {
            events: VecDeque::default(),
            max_id: 0,
            receivers: HashMap::default(),
            free_ids: Vec::default(),
        }
    }
}

impl<E> fmt::Debug for EventQueue<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventQueue {{ #events: {}, #receivers: {} }}", self.events.len(), self.receivers.len())
    }
}

impl<E> Resource for EventQueue<E> where E: 'static {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct MockEvent(usize);

    #[test]
    fn default_event_queue() {
        let _: EventQueue<MockEvent> = EventQueue::default();
    }

    #[test]
    fn send_no_receivers() {
        let mut q: EventQueue<MockEvent> = EventQueue::default();
        assert_eq!(q.len(), 0);
        q.send(MockEvent(0));
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn subscribe() {
        let mut q: EventQueue<MockEvent> = EventQueue::default();
        let s: ReceiverId<MockEvent> = q.subscribe();
        assert_eq!(q.subscribers(), 1);
        q.unsubscribe(s);
        assert_eq!(q.subscribers(), 0);
    }

    #[test]
    fn send_one_receiver() {
        let mut q: EventQueue<MockEvent> = EventQueue::default();
        assert_eq!(q.subscribers(), 0);

        let s = q.subscribe();

        q.send(MockEvent(0));
        assert_eq!(q.len(), 1);

        let evs: Vec<MockEvent> = q.receive(&s);
        assert_eq!(evs, vec![MockEvent(0)]);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn send_two_concurrent_receivers() {
        let mut q: EventQueue<MockEvent> = EventQueue::default();

        // Subscribe both receivers
        let s = q.subscribe();
        let t = q.subscribe();
        assert_eq!(q.subscribers(), 2);

        // Send first event
        q.send(MockEvent(0));
        assert_eq!(q.len(), 1);

        // Receive with both receivers
        let s_evs: Vec<MockEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![MockEvent(0)]);
        assert_eq!(q.len(), 1);
        let t_evs: Vec<MockEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![MockEvent(0)]);
        assert_eq!(q.len(), 0);

        // Send two additional events
        q.send(MockEvent(1));
        q.send(MockEvent(2));
        assert_eq!(q.len(), 2);

        // Receive with both receivers
        let s_evs: Vec<MockEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![MockEvent(2), MockEvent(1)]);
        assert_eq!(q.len(), 2);
        let t_evs: Vec<MockEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![MockEvent(2), MockEvent(1)]);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn send_two_interleaved_receivers() {
        let mut q: EventQueue<MockEvent> = EventQueue::default();

        // Subscribe with the first receiver
        let s = q.subscribe();

        // Send the first event
        q.send(MockEvent(0));
        assert_eq!(q.len(), 1);

        // Subscribe with the second receiver
        let t = q.subscribe();

        // Send the second event
        q.send(MockEvent(1));
        assert_eq!(q.len(), 2);

        // Receive with the first receiver
        let s_evs: Vec<MockEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![MockEvent(1), MockEvent(0)]);
        assert_eq!(q.len(), 1);

        // Send the third event
        q.send(MockEvent(2));
        assert_eq!(q.len(), 2);

        let s_evs: Vec<MockEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![MockEvent(2)]);
        assert_eq!(q.len(), 2);

        let t_evs: Vec<MockEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![MockEvent(2), MockEvent(1)]);
        assert_eq!(q.len(), 0);
    }
}
