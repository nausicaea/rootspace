use std::collections::VecDeque;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::Duration;
use system::SystemTrait;
use loop_stage::LoopStage;
use event::EventTrait;
use database::DatabaseTrait;

pub trait WorldTrait {
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    fn dynamic_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    fn render(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    fn handle_events(&mut self) -> Result<bool, Error>;
}

pub struct World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<E, A, D> {
    systems: Vec<S>,
    database: D,
    auxiliary: A,
    event_queue: VecDeque<E>,
}

impl<A, D, E, S> World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<E, A, D> {
    pub fn add_system<T: Into<S>>(&mut self, system: T) {
        self.systems.push(system.into());
    }
    pub fn dispatch_later(&mut self, event: E) {
        self.event_queue.push_back(event);
    }
    pub fn dispatch_now(&mut self, event: E) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) {
                if event.matches_filter(system.get_event_filter()) {
                    system.handle_event(&mut self.database, &mut self.auxiliary, &event);
                }
            }
        }
        Ok(())
    }
}

impl<A, D, E, S> Default for World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<E, A, D> {
    fn default() -> Self {
        World {
            systems: Default::default(),
            database: Default::default(),
            auxiliary: Default::default(),
            event_queue: Default::default(),
        }
    }
}

impl<A, D, E, S> Debug for World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<E, A, D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "World(systems: {})", self.systems.len())
    }
}

impl<A, D, E, S> WorldTrait for World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<E, A, D> {
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::UPDATE) {
                system.update(&mut self.database, &mut self.auxiliary, time, delta_time);
            }
        }
        Ok(())
    }
    fn dynamic_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE) {
                system.dynamic_update(&mut self.database, &mut self.auxiliary, time, delta_time);
            }
        }
        Ok(())
    }
    fn render(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::RENDER) {
                system.render(&self.database, &mut self.auxiliary, time, delta_time);
            }
        }
        Ok(())
    }
    fn handle_events(&mut self) -> Result<bool, Error> {
        let events = self.event_queue.iter().cloned().collect::<Vec<_>>();
        self.event_queue.clear();

        for event in events {
            self.dispatch_now(event)?;
        }
        warn!("Issuing the signal to terminate the main loop.");
        Ok(false)
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unspecified error")]
    UnspecifiedError,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub enum MockEvent {
        TestEventA(String),
        TestEventB(u32),
    }

    impl MockEvent {
        fn as_flag(&self) -> MockEventFlag {
            match *self {
                MockEvent::TestEventA(_) => MockEventFlag::TEST_EVENT_A,
                MockEvent::TestEventB(_) => MockEventFlag::TEST_EVENT_B,
            }
        }
    }

    bitflags! {
        #[derive(Default)]
        pub struct MockEventFlag: u8 {
            const TEST_EVENT_A = 0x01;
            const TEST_EVENT_B = 0x02;
        }
    }

    impl EventTrait for MockEvent {
        type EventFlag = MockEventFlag;

        fn matches_filter(&self, flag: Self::EventFlag) -> bool {
            flag.contains(self.as_flag())
        }
    }

    #[derive(Default, Clone, Debug, PartialEq)]
    struct MockAuxiliary;

    #[derive(Default, Clone, Debug, PartialEq)]
    struct MockDatabase;

    impl DatabaseTrait for MockDatabase {
    }

    #[derive(Clone, PartialEq, Debug)]
    struct MockSystem<E, A, D> where E: EventTrait, D: DatabaseTrait {
        update_calls: usize,
        update_arguments: Vec<(Duration, Duration)>,
        dynamic_update_calls: usize,
        dynamic_update_arguments: Vec<(Duration, Duration)>,
        render_calls: usize,
        render_arguments: Vec<(Duration, Duration)>,
        handle_event_calls: usize,
        handle_event_arguments: Vec<E>,
        stage_filter: LoopStage,
        event_filter: E::EventFlag,
        phantom_a: PhantomData<A>,
        phantom_b: PhantomData<D>,
    }

    impl<E, A, D> MockSystem<E, A, D> where E: EventTrait, D: DatabaseTrait {
        pub fn new(stage_filter: LoopStage, event_filter: E::EventFlag) -> Self {
            MockSystem {
                stage_filter: stage_filter,
                event_filter: event_filter,
                .. Default::default()
            }
        }
    }

    impl<E, A, D> From<(LoopStage, E::EventFlag)> for MockSystem<E, A, D> where E: EventTrait, D: DatabaseTrait {
        fn from(value: (LoopStage, E::EventFlag)) -> Self {
            MockSystem::new(value.0, value.1)
        }
    }

    impl<E, A, D> Default for MockSystem<E, A, D> where E: EventTrait, D: DatabaseTrait {
        fn default() -> Self {
            MockSystem {
                update_calls: 0,
                update_arguments: Vec::new(),
                dynamic_update_calls: 0,
                dynamic_update_arguments: Vec::new(),
                render_calls: 0,
                render_arguments: Vec::new(),
                handle_event_calls: 0,
                handle_event_arguments: Vec::new(),
                stage_filter: LoopStage::empty(),
                event_filter: Default::default(),
                phantom_a: Default::default(),
                phantom_b: Default::default(),
            }
        }
    }

    impl<E, A, D> SystemTrait<E, A, D> for MockSystem<E, A, D> where E: EventTrait, D: DatabaseTrait {
        fn get_stage_filter(&self) -> LoopStage {
            self.stage_filter
        }
        fn get_event_filter(&self) -> E::EventFlag {
            self.event_filter
        }
        fn update(&mut self, _db: &mut D, _aux: &mut A, time: &Duration, delta_time: &Duration) {
            self.update_arguments.push((*time, *delta_time));
            self.update_calls += 1;
        }
        fn dynamic_update(&mut self, _db: &mut D, _aux: &mut A, time: &Duration, delta_time: &Duration) {
            self.dynamic_update_arguments.push((*time, *delta_time));
            self.dynamic_update_calls += 1;
        }
        fn render(&mut self, _db: &D, _aux: &mut A, time: &Duration, delta_time: &Duration) {
            self.render_arguments.push((*time, *delta_time));
            self.render_calls += 1;
        }
        fn handle_event(&mut self, _db: &mut D, _aux: &mut A, event: &E) {
            self.handle_event_arguments.push(event.clone());
            self.handle_event_calls += 1;
        }
    }

    fn create_populated_world() -> World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockEvent, MockAuxiliary, MockDatabase>> {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockEvent, MockAuxiliary, MockDatabase>> = World::default();
        w.systems = [
            MockSystem::new(LoopStage::UPDATE, MockEventFlag::empty()),
            MockSystem::new(LoopStage::UPDATE, MockEventFlag::empty()),
            MockSystem::new(LoopStage::DYNAMIC_UPDATE, MockEventFlag::empty()),
            MockSystem::new(LoopStage::DYNAMIC_UPDATE, MockEventFlag::empty()),
            MockSystem::new(LoopStage::RENDER, MockEventFlag::empty()),
            MockSystem::new(LoopStage::RENDER, MockEventFlag::empty()),
            MockSystem::new(LoopStage::HANDLE_EVENTS, MockEventFlag::TEST_EVENT_A),
            MockSystem::new(LoopStage::HANDLE_EVENTS, MockEventFlag::TEST_EVENT_B),
        ].iter().cloned().collect();
        w.event_queue = [
            MockEvent::TestEventA("lala-land".into()),
            MockEvent::TestEventB(100),
        ].iter().cloned().collect();
        w
    }

    #[test]
    fn add_system() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockEvent, MockAuxiliary, MockDatabase>> = World::default();
        assert!(w.systems.is_empty());
        let sys = MockSystem::new(LoopStage::UPDATE, MockEventFlag::empty());
        w.add_system(sys.clone());
        assert_eq!(w.systems.len(), 1);
        assert_eq!(w.systems.last().unwrap(), &sys);
        let into_sys = (LoopStage::UPDATE, MockEventFlag::empty());
        w.add_system(into_sys);
        assert_eq!(w.systems.len(), 2);
        assert_eq!(w.systems.last().unwrap(), &sys);
    }
    #[test]
    fn dispatch_later() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockEvent, MockAuxiliary, MockDatabase>> = World::default();
        assert!(w.event_queue.is_empty());
        let evt = MockEvent::TestEventA("hello".into());
        w.dispatch_later(evt.clone());
        assert_eq!(w.event_queue.len(), 1);
        assert_eq!(w.event_queue.front().unwrap(), &evt);
    }
    #[test]
    fn update_calls() {
        let mut w = create_populated_world();
        let r = w.update(&Duration::default(), &Duration::default());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::UPDATE)).all(|s| s.update_calls == 1));
        assert!(w.systems.iter().filter(|s| !s.get_stage_filter().contains(LoopStage::UPDATE)).all(|s| s.update_calls == 0));
    }
    #[test]
    fn update_arguments() {
        let mut w = create_populated_world();
        w.update(&Duration::new(1, 0), &Duration::new(0, 1)).unwrap();

        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::UPDATE)).all(|s| s.update_arguments.iter().all(|&(t, dt)| {
            t == Duration::new(1, 0) && dt == Duration::new(0, 1)
        })));
    }
    #[test]
    fn dynamic_update_calls() {
        let mut w = create_populated_world();
        let r = w.dynamic_update(&Duration::default(), &Duration::default());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE)).all(|s| s.dynamic_update_calls == 1));
        assert!(w.systems.iter().filter(|s| !s.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE)).all(|s| s.dynamic_update_calls == 0));
    }
    #[test]
    fn dynamic_update_arguments() {
        let mut w = create_populated_world();
        w.dynamic_update(&Duration::new(1, 0), &Duration::new(0, 1)).unwrap();

        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE)).all(|s| s.dynamic_update_arguments.iter().all(|&(t, dt)| {
            t == Duration::new(1, 0) && dt == Duration::new(0, 1)
        })));
    }
    #[test]
    fn render_calls() {
        let mut w = create_populated_world();
        let r = w.render(&Duration::default(), &Duration::default());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::RENDER)).all(|s| s.render_calls == 1));
        assert!(w.systems.iter().filter(|s| !s.get_stage_filter().contains(LoopStage::RENDER)).all(|s| s.render_calls == 0));
    }
    #[test]
    fn render_arguments() {
        let mut w = create_populated_world();
        w.render(&Duration::new(1, 0), &Duration::new(0, 1)).unwrap();

        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::RENDER)).all(|s| s.render_arguments.iter().all(|&(t, dt)| {
            t == Duration::new(1, 0) && dt == Duration::new(0, 1)
        })));
    }
    #[test]
    fn handle_event_calls() {
        let mut w = create_populated_world();
        let r = w.handle_events();

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) && s.get_event_filter().contains(MockEventFlag::TEST_EVENT_A)).all(|s| s.handle_event_calls == 1));
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) && s.get_event_filter().contains(MockEventFlag::TEST_EVENT_B)).all(|s| s.handle_event_calls == 1));
        assert!(w.systems.iter().filter(|s| !s.get_stage_filter().contains(LoopStage::HANDLE_EVENTS)).all(|s| s.handle_event_calls == 0));
    }
    #[test]
    fn handle_event_arguments() {
        let mut w = create_populated_world();
        w.handle_events().unwrap();

        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) && s.get_event_filter().contains(MockEventFlag::TEST_EVENT_A)).all(|s| s.handle_event_arguments.iter().all(|e| e.as_flag() == MockEventFlag::TEST_EVENT_A)));
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) && s.get_event_filter().contains(MockEventFlag::TEST_EVENT_B)).all(|s| s.handle_event_arguments.iter().all(|e| e.as_flag() == MockEventFlag::TEST_EVENT_B)));
    }
}
