//! Provides facilities to define and manage events.

use crate::resource::Resource;
#[cfg(any(test, debug_assertions))]
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    marker::PhantomData,
};
use std::any::type_name;

/// A handle that allows a receiver to receive events from the related event queue.
#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiverId<E> {
    id: usize,
    #[serde(skip, default = "PhantomData::default")]
    _e: PhantomData<E>,
}

impl<E> std::fmt::Debug for ReceiverId<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ReceiverId<{}> {{ id: {:?} }}", std::any::type_name::<E>(), self.id)
    }
}

impl<E> ReceiverId<E> {
    fn new(id: usize) -> Self {
        ReceiverId {
            id,
            _e: PhantomData::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct ReceiverState<E> {
    read: usize,
    received: usize,
    #[serde(skip, default = "PhantomData::default")]
    _e: PhantomData<E>,
}

impl<E> ReceiverState<E> {
    fn reset(&mut self) {
        self.read = 0;
        self.received = 0;
    }
}

impl<E> std::fmt::Debug for ReceiverState<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ReceiverState<{}> {{ read: {:?}, received: {:?} }}", std::any::type_name::<E>(), self.read, self.received)
    }
}

impl<E> Default for ReceiverState<E> {
    fn default() -> Self {
        ReceiverState {
            read: 0,
            received: 0,
            _e: PhantomData::default(),
        }
    }
}

/// An `EventQueue` contains a queue of events and provides rudimentary facilities of retrieving
/// those events.
#[derive(Serialize, Deserialize)]
pub struct EventQueue<E> {
    events: VecDeque<E>,
    receivers: HashMap<usize, ReceiverState<E>>,
    max_id: usize,
    free_ids: Vec<usize>,
}

impl<E> std::fmt::Debug for EventQueue<E>
where
    E: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "EventQueue<{}> {{ events: {:?}, receivers: {:?}, max_id: {:?}, free_ids: {:?} }}", std::any::type_name::<E>(), self.events, self.receivers, self.max_id, self.free_ids)
    }
}

impl<E> EventQueue<E>
where
    E: Clone,
{
    /// Subscribe to this event queue.
    pub fn subscribe<T>(&mut self) -> ReceiverId<E> {
        let _t: PhantomData<T> = PhantomData::default();

        let id = if let Some(id) = self.free_ids.pop() {
            id
        } else {
            let tmp = self.max_id;
            self.max_id += 1;
            tmp
        };

        self.receivers.insert(id, ReceiverState::default());

        #[cfg(any(test, debug_assertions))]
        debug!(
            "Adding subscriber {} to queue {}",
            type_name::<T>(),
            type_name::<Self>()
        );
        ReceiverId::new(id)
    }

    /// Unsubscribe from this event queue.
    pub fn unsubscribe(&mut self, id: ReceiverId<E>) {
        self.receivers.remove(&id.id);
        self.free_ids.push(id.id);
    }

    /// Renew an existing subscription to this event queue
    pub fn renew(&mut self, id: Option<ReceiverId<E>>) -> ReceiverId<E> {
        if let Some(recv) = id {
            self.unsubscribe(recv);
        }

        self.subscribe::<Self>()
    }

    /// Return `true` if the receiver is subscribed to this
    /// [`EventQueue<T>`](crate::event_queue::EventQueue), `false` otherwise.
    pub fn is_subscribed(&self, id: &ReceiverId<E>) -> bool {
        self.receivers.contains_key(&id.id)
    }

    /// Send an event into the queue.
    pub fn send(&mut self, event: E) {
        if !self.receivers.is_empty() {
            self.events.push_front(event);
            self.receivers.values_mut().for_each(|s| s.received += 1);
        }
    }

    /// Receive all unread events from the queue.
    pub fn receive(&mut self, id: &ReceiverId<E>) -> Vec<E> {
        // Obtain all unread events for the current receiver
        let events = &self.events;
        let evs: Vec<E> = self
            .receivers
            .get_mut(&id.id)
            .filter(|s| s.read < s.received)
            .map(|s| {
                let total = events.len();
                let unread = s.received - s.read;
                s.read += unread;
                events
                    .iter()
                    .rev()
                    .skip(total - unread)
                    .take(unread)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        // Delete all events that have been read by all receivers
        let max_unread = self
            .receivers
            .values()
            .map(|s| s.received - s.read)
            .max()
            .unwrap_or_default();
        self.events.truncate(max_unread);

        // If the event queue is empty, or all events have been read by all receivers, reset the
        // counters for each receiver
        if self.events.is_empty() {
            self.receivers.values_mut().for_each(|s| s.reset());
        }

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
}

impl<E> Resource for EventQueue<E> where E: fmt::Debug + 'static {}

impl<E> Default for EventQueue<E> {
    fn default() -> Self {
        EventQueue {
            events: VecDeque::default(),
            receivers: HashMap::default(),
            max_id: 0,
            free_ids: Vec::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct MockEvent(usize);

    #[test]
    fn default_event_queue() {
        let _: EventQueue<MockEvent> = Default::default();
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
        assert_eq!(q.subscribers(), 0);
        let s: ReceiverId<MockEvent> = q.subscribe();
        assert_eq!(q.subscribers(), 1);
        q.unsubscribe(s);
        assert_eq!(q.subscribers(), 0);
    }

    #[test]
    fn send_one_receiver() {
        let mut q: EventQueue<MockEvent> = EventQueue::default();

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
        assert_eq!(s_evs, vec![MockEvent(1), MockEvent(2)]);
        assert_eq!(q.len(), 2);
        let t_evs: Vec<MockEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![MockEvent(1), MockEvent(2)]);
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
        assert_eq!(s_evs, vec![MockEvent(0), MockEvent(1)]);
        assert_eq!(q.len(), 1);

        // Send the third event
        q.send(MockEvent(2));
        assert_eq!(q.len(), 2);

        let s_evs: Vec<MockEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![MockEvent(2)]);
        assert_eq!(q.len(), 2);

        let t_evs: Vec<MockEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![MockEvent(1), MockEvent(2)]);
        assert_eq!(q.len(), 0);
    }
}
