use std::collections::VecDeque;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::Duration;
use failure::Error;
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

pub struct World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<A, D, E> {
    systems: Vec<S>,
    database: D,
    auxiliary: A,
    event_queue: VecDeque<E>,
}

impl<A, D, E, S> World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<A, D, E> {
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
                    system.handle_event(&mut self.database, &mut self.auxiliary, &event)?;
                }
            }
        }
        Ok(())
    }
}

impl<A, D, E, S> Default for World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<A, D, E> {
    fn default() -> Self {
        World {
            systems: Default::default(),
            database: Default::default(),
            auxiliary: Default::default(),
            event_queue: Default::default(),
        }
    }
}

impl<A, D, E, S> Debug for World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<A, D, E> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "World(systems: {})", self.systems.len())
    }
}

impl<A, D, E, S> WorldTrait for World<A, D, E, S> where A: Default, D: DatabaseTrait, E: EventTrait, S: SystemTrait<A, D, E> {
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::UPDATE) {
                system.update(&mut self.database, &mut self.auxiliary, time, delta_time)?;
            }
        }
        Ok(())
    }
    fn dynamic_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE) {
                system.dynamic_update(&mut self.database, &mut self.auxiliary, time, delta_time)?;
            }
        }
        Ok(())
    }
    fn render(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::RENDER) {
                system.render(&self.database, &mut self.auxiliary, time, delta_time)?;
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
    struct MockSystem<A, D, E> where E: EventTrait, D: DatabaseTrait {
        stage_filter: LoopStage,
        event_filter: E::EventFlag,
        error_out: bool,
        update_calls: usize,
        update_arguments: Vec<(Duration, Duration)>,
        dynamic_update_calls: usize,
        dynamic_update_arguments: Vec<(Duration, Duration)>,
        render_calls: usize,
        render_arguments: Vec<(Duration, Duration)>,
        handle_event_calls: usize,
        handle_event_arguments: Vec<E>,
        phantom_a: PhantomData<A>,
        phantom_b: PhantomData<D>,
    }

    impl<A, D, E> MockSystem<A, D, E> where E: EventTrait, D: DatabaseTrait {
        pub fn new(stage_filter: LoopStage, event_filter: E::EventFlag, error_out: bool) -> Self {
            MockSystem {
                stage_filter: stage_filter,
                event_filter: event_filter,
                error_out: error_out,
                .. Default::default()
            }
        }
    }

    impl<A, D, E> From<(LoopStage, E::EventFlag, bool)> for MockSystem<A, D, E> where E: EventTrait, D: DatabaseTrait {
        fn from(value: (LoopStage, E::EventFlag, bool)) -> Self {
            MockSystem::new(value.0, value.1, value.2)
        }
    }

    impl<A, D, E> Default for MockSystem<A, D, E> where E: EventTrait, D: DatabaseTrait {
        fn default() -> Self {
            MockSystem {
                stage_filter: Default::default(),
                event_filter: Default::default(),
                error_out: Default::default(),
                update_calls: 0,
                update_arguments: Vec::new(),
                dynamic_update_calls: 0,
                dynamic_update_arguments: Vec::new(),
                render_calls: 0,
                render_arguments: Vec::new(),
                handle_event_calls: 0,
                handle_event_arguments: Vec::new(),
                phantom_a: Default::default(),
                phantom_b: Default::default(),
            }
        }
    }

    impl<A, D, E> SystemTrait<A, D, E> for MockSystem<A, D, E> where E: EventTrait, D: DatabaseTrait {
        fn get_stage_filter(&self) -> LoopStage {
            self.stage_filter
        }
        fn get_event_filter(&self) -> E::EventFlag {
            self.event_filter
        }
        fn update(&mut self, _db: &mut D, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.update_arguments.push((*time, *delta_time));
            self.update_calls += 1;
            if self.error_out {
                Err(format_err!("MockSystem.update() had an error"))
            } else {
                Ok(())
            }
        }
        fn dynamic_update(&mut self, _db: &mut D, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.dynamic_update_arguments.push((*time, *delta_time));
            self.dynamic_update_calls += 1;
            if self.error_out {
                Err(format_err!("MockSystem.dynamic_update() had an error"))
            } else {
                Ok(())
            }
        }
        fn render(&mut self, _db: &D, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.render_arguments.push((*time, *delta_time));
            self.render_calls += 1;
            if self.error_out {
                Err(format_err!("MockSystem.render() had an error"))
            } else {
                Ok(())
            }
        }
        fn handle_event(&mut self, _db: &mut D, _aux: &mut A, event: &E) -> Result<(), Error> {
            self.handle_event_arguments.push(event.clone());
            self.handle_event_calls += 1;
            if self.error_out {
                Err(format_err!("MockSystem.handle_event() had an error"))
            } else {
                Ok(())
            }
        }
    }

    fn create_populated_world() -> World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> = World::default();
        w.systems = [
            MockSystem::new(LoopStage::UPDATE, MockEventFlag::empty(), false),
            MockSystem::new(LoopStage::UPDATE, MockEventFlag::empty(), false),
            MockSystem::new(LoopStage::DYNAMIC_UPDATE, MockEventFlag::empty(), false),
            MockSystem::new(LoopStage::DYNAMIC_UPDATE, MockEventFlag::empty(), false),
            MockSystem::new(LoopStage::RENDER, MockEventFlag::empty(), false),
            MockSystem::new(LoopStage::RENDER, MockEventFlag::empty(), false),
            MockSystem::new(LoopStage::HANDLE_EVENTS, MockEventFlag::TEST_EVENT_A, false),
            MockSystem::new(LoopStage::HANDLE_EVENTS, MockEventFlag::TEST_EVENT_B, false),
        ].iter().cloned().collect();
        w.event_queue = [
            MockEvent::TestEventA("lala-land".into()),
            MockEvent::TestEventB(100),
        ].iter().cloned().collect();
        w
    }

    #[test]
    fn add_system() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> = World::default();
        assert!(w.systems.is_empty());
        let sys = MockSystem::new(LoopStage::UPDATE, MockEventFlag::empty(), false);
        w.add_system(sys.clone());
        assert_eq!(w.systems.len(), 1);
        assert_eq!(w.systems.last().unwrap(), &sys);
        let into_sys = (LoopStage::UPDATE, MockEventFlag::empty(), false);
        w.add_system(into_sys);
        assert_eq!(w.systems.len(), 2);
        assert_eq!(w.systems.last().unwrap(), &sys);
    }
    #[test]
    fn dispatch_later() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> = World::default();
        assert!(w.event_queue.is_empty());
        let evt = MockEvent::TestEventA("hello".into());
        w.dispatch_later(evt.clone());
        assert_eq!(w.event_queue.len(), 1);
        assert_eq!(w.event_queue.front().unwrap(), &evt);
    }
    #[test]
    fn dispatch_now() {
        let mut w = create_populated_world();
        let evt = MockEvent::TestEventA("hello".into());
        let r = w.dispatch_now(evt.clone());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) && s.get_event_filter().contains(MockEventFlag::TEST_EVENT_A)).all(|s| s.handle_event_calls == 1));
        assert!(w.systems.iter().filter(|s| s.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) && s.get_event_filter().contains(MockEventFlag::TEST_EVENT_A)).all(|s| s.handle_event_arguments.iter().all(|e| e == &evt)));
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
    fn update_error() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> = World::default();
        w.systems.push(MockSystem::new(LoopStage::UPDATE, MockEventFlag::empty(), true));
        let r = w.update(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
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
    fn dynamic_update_error() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> = World::default();
        w.systems.push(MockSystem::new(LoopStage::DYNAMIC_UPDATE, MockEventFlag::empty(), true));
        let r = w.dynamic_update(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
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
    fn render_error() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> = World::default();
        w.systems.push(MockSystem::new(LoopStage::RENDER, MockEventFlag::empty(), true));
        let r = w.render(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
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
    #[test]
    fn handle_event_error() {
        let mut w: World<MockAuxiliary, MockDatabase, MockEvent, MockSystem<MockAuxiliary, MockDatabase, MockEvent>> = World::default();
        w.systems.push(MockSystem::new(LoopStage::HANDLE_EVENTS, MockEventFlag::TEST_EVENT_A, true));
        w.event_queue.push_back(MockEvent::TestEventA("hello".into()));
        let r = w.handle_events();
        assert!(r.is_err());
    }
}
