//! Provides facilities to define and manage events.

use std::{
    any::type_name,
    collections::{HashMap, VecDeque},
    fmt,
};

use log::debug;
use receiver_state::ReceiverState;
use serde::{Deserialize, Serialize};

use self::receiver_id::ReceiverId;
use crate::{resource::Resource, with_dependencies::WithDependencies};

pub mod receiver_id;
pub mod receiver_state;

/// An `EventQueue` contains a queue of events and provides rudimentary facilities of retrieving
/// those events. The transmitted event `E` must be `Clone` because of the one-to-many
/// relationships (i.e. single sender, multiple receivers).
#[derive(PartialEq, Serialize, Deserialize)]
pub struct EventQueue<E> {
    #[serde(default = "VecDeque::default", skip_serializing_if = "VecDeque::is_empty")]
    events: VecDeque<E>,
    #[serde(default = "HashMap::default", skip_serializing_if = "HashMap::is_empty")]
    receivers: HashMap<usize, ReceiverState<E>>,
    max_id: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    free_ids: Vec<usize>,
}

impl<E> EventQueue<E>
where
    E: Clone,
{
    /// Subscribe to this event queue.
    pub fn subscribe<T>(&mut self) -> ReceiverId<E> {
        let id = if let Some(id) = self.free_ids.pop() {
            id
        } else {
            let tmp = self.max_id;
            self.max_id += 1;
            tmp
        };

        self.receivers.insert(id, ReceiverState::new(id));

        let stnt = type_name::<T>();
        let stns = type_name::<Self>();
        debug!("Adding subscriber {} to queue {}", stnt, stns);
        ReceiverId::new(id)
    }

    /// Unsubscribe from this event queue.
    pub fn unsubscribe(&mut self, id: ReceiverId<E>) {
        self.receivers.remove(&id.id());
        self.free_ids.push(id.id());
    }

    /// Return `true` if the receiver is subscribed to this
    /// [`EventQueue<T>`](crate::event_queue::EventQueue), `false` otherwise.
    pub fn is_subscribed(&self, id: &ReceiverId<E>) -> bool {
        self.receivers.contains_key(&id.id())
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
            .get_mut(&id.id())
            .filter(|s| s.read < s.received)
            .map(|s| {
                let total = events.len();
                let unread = s.received - s.read;
                s.read += unread;
                events.iter().rev().skip(total - unread).take(unread).cloned().collect()
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

    /// Return `true` if there are no queued events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
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

impl<D, E> WithDependencies<D> for EventQueue<E> {
    fn with_deps(_: &D) -> Result<Self, anyhow::Error> {
        Ok(EventQueue::default())
    }
}

impl<E> std::fmt::Debug for EventQueue<E>
where
    E: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "EventQueue {{ events: {:?}, receivers: {:?}, max_id: {:?}, free_ids: {:?} }}",
            self.events, self.receivers, self.max_id, self.free_ids,
        )
    }
}

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
    use serde_test::{assert_tokens, Token};

    use super::*;
    use crate::{End, Reg, ResourceRegistry, World};

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    struct TestEvent(usize);

    #[test]
    fn event_queue_reg_macro() {
        type _RR = Reg![EventQueue<usize>];
    }

    #[test]
    fn event_queue_resource_registry() {
        let _rr = ResourceRegistry::push(End, EventQueue::<usize>::default());
    }

    #[test]
    fn event_queue_world() {
        let _w = World::with_dependencies::<Reg![EventQueue<usize>], Reg![], Reg![], Reg![], _>(&()).unwrap();
    }

    #[test]
    fn default_event_queue() {
        let _: EventQueue<TestEvent> = Default::default();
    }

    #[test]
    fn send_no_receivers() {
        let mut q: EventQueue<TestEvent> = EventQueue::default();
        assert_eq!(q.len(), 0);
        q.send(TestEvent(0));
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn subscribe() {
        let mut q: EventQueue<TestEvent> = EventQueue::default();
        assert_eq!(q.subscribers(), 0);
        let s: ReceiverId<TestEvent> = q.subscribe::<()>();
        assert_eq!(q.subscribers(), 1);
        q.unsubscribe(s);
        assert_eq!(q.subscribers(), 0);
    }

    #[test]
    fn send_one_receiver() {
        let mut q: EventQueue<TestEvent> = EventQueue::default();

        let s = q.subscribe::<()>();

        q.send(TestEvent(0));
        assert_eq!(q.len(), 1);

        let evs: Vec<TestEvent> = q.receive(&s);
        assert_eq!(evs, vec![TestEvent(0)]);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn send_two_concurrent_receivers() {
        let mut q: EventQueue<TestEvent> = EventQueue::default();

        // Subscribe both receivers
        let s = q.subscribe::<()>();
        let t = q.subscribe::<()>();
        assert_eq!(q.subscribers(), 2);

        // Send first event
        q.send(TestEvent(0));
        assert_eq!(q.len(), 1);

        // Receive with both receivers
        let s_evs: Vec<TestEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![TestEvent(0)]);
        assert_eq!(q.len(), 1);
        let t_evs: Vec<TestEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![TestEvent(0)]);
        assert_eq!(q.len(), 0);

        // Send two additional events
        q.send(TestEvent(1));
        q.send(TestEvent(2));
        assert_eq!(q.len(), 2);

        // Receive with both receivers
        let s_evs: Vec<TestEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![TestEvent(1), TestEvent(2)]);
        assert_eq!(q.len(), 2);
        let t_evs: Vec<TestEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![TestEvent(1), TestEvent(2)]);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn send_two_interleaved_receivers() {
        let mut q: EventQueue<TestEvent> = EventQueue::default();

        // Subscribe with the first receiver
        let s = q.subscribe::<()>();

        // Send the first event
        q.send(TestEvent(0));
        assert_eq!(q.len(), 1);

        // Subscribe with the second receiver
        let t = q.subscribe::<()>();

        // Send the second event
        q.send(TestEvent(1));
        assert_eq!(q.len(), 2);

        // Receive with the first receiver
        let s_evs: Vec<TestEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![TestEvent(0), TestEvent(1)]);
        assert_eq!(q.len(), 1);

        // Send the third event
        q.send(TestEvent(2));
        assert_eq!(q.len(), 2);

        let s_evs: Vec<TestEvent> = q.receive(&s);
        assert_eq!(s_evs, vec![TestEvent(2)]);
        assert_eq!(q.len(), 2);

        let t_evs: Vec<TestEvent> = q.receive(&t);
        assert_eq!(t_evs, vec![TestEvent(1), TestEvent(2)]);
        assert_eq!(q.len(), 0);
    }

    #[test]
    #[ignore]
    fn event_queue_serde() {
        let mut eq = EventQueue::<TestEvent>::default();
        eq.send(TestEvent(0));
        let _r1 = eq.subscribe::<()>();
        eq.send(TestEvent(1));
        let r2 = eq.subscribe::<()>();
        eq.send(TestEvent(2));
        let _r3 = eq.subscribe::<()>();
        eq.send(TestEvent(3));
        eq.unsubscribe(r2);

        assert_tokens(
            &eq,
            &[
                Token::Struct {
                    name: "EventQueue",
                    len: 4,
                },
                Token::Str("events"),
                Token::Seq { len: Some(3) },
                Token::NewtypeStruct { name: "TestEvent" },
                Token::U64(3),
                Token::NewtypeStruct { name: "TestEvent" },
                Token::U64(2),
                Token::NewtypeStruct { name: "TestEvent" },
                Token::U64(1),
                Token::SeqEnd,
                Token::Str("receivers"),
                Token::Map { len: Some(2) },
                Token::U64(0),
                Token::Tuple { len: 3 },
                Token::U64(0),
                Token::U64(0),
                Token::U64(3),
                Token::TupleEnd,
                Token::U64(2),
                Token::Tuple { len: 3 },
                Token::U64(2),
                Token::U64(0),
                Token::U64(1),
                Token::TupleEnd,
                Token::MapEnd,
                Token::Str("max_id"),
                Token::U64(3),
                Token::Str("free_ids"),
                Token::Seq { len: Some(1) },
                Token::U64(1),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}
