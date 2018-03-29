use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use system::SystemTrait;
use loop_stage::LoopStage;
use event::{EventTrait, EventManagerTrait};
use database::DatabaseTrait;

pub trait WorldTrait {
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    fn dynamic_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    fn render(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    fn handle_events(&mut self) -> Result<bool, Error>;
}

pub struct World<H, A, D, E, S>
where
    H: EventManagerTrait<E>,
    A: Default,
    D: DatabaseTrait,
    E: EventTrait,
    S: SystemTrait<H, A, D, E>,
{
    systems: Vec<S>,
    database: D,
    auxiliary: A,
    event_manager: H,
    phantom: PhantomData<E>,
}

impl<H, A, D, E, S> World<H, A, D, E, S>
where
    H: EventManagerTrait<E>,
    A: Default,
    D: DatabaseTrait,
    E: EventTrait,
    S: SystemTrait<H, A, D, E>,
{
    pub fn add_system<T: Into<S>>(&mut self, system: T) {
        self.systems.push(system.into());
    }
}

impl<H, A, D, E, S> Default for World<H, A, D, E, S>
where
    H: EventManagerTrait<E>,
    A: Default,
    D: DatabaseTrait,
    E: EventTrait,
    S: SystemTrait<H, A, D, E>,
{
    fn default() -> Self {
        World {
            systems: Default::default(),
            database: Default::default(),
            auxiliary: Default::default(),
            event_manager: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<H, A, D, E, S> Debug for World<H, A, D, E, S>
where
    H: EventManagerTrait<E>,
    A: Default,
    D: DatabaseTrait,
    E: EventTrait,
    S: SystemTrait<H, A, D, E>,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "World(systems: {})", self.systems.len())
    }
}

impl<H, A, D, E, S> WorldTrait for World<H, A, D, E, S>
where
    H: EventManagerTrait<E>,
    A: Default,
    D: DatabaseTrait,
    E: EventTrait,
    S: SystemTrait<H, A, D, E>,
{
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::UPDATE) {
                system.update(&mut self.database, &mut self.event_manager, &mut self.auxiliary, time, delta_time)?;
            }
        }
        Ok(())
    }
    fn dynamic_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE) {
                system.dynamic_update(&mut self.database, &mut self.event_manager, &mut self.auxiliary, time, delta_time)?;
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
        let systems = &mut self.systems;
        let database = &mut self.database;
        let auxiliary = &mut self.auxiliary;
        self.event_manager.handle_events(|mgr, event| {
            for system in systems.iter_mut() {
                if system.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) {
                    if event.matches_filter(system.get_event_filter()) {
                        system.handle_event(database, mgr, auxiliary, event)?;
                    }
                }
            }
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::marker::PhantomData;
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub enum MockEvt {
        TestEventA(String),
        TestEventB(u32),
    }

    impl MockEvt {
        fn as_flag(&self) -> MockEvtFlag {
            match *self {
                MockEvt::TestEventA(_) => MockEvtFlag::TEST_EVENT_A,
                MockEvt::TestEventB(_) => MockEvtFlag::TEST_EVENT_B,
            }
        }
    }

    bitflags! {
        #[derive(Default)]
        pub struct MockEvtFlag: u8 {
            const TEST_EVENT_A = 0x01;
            const TEST_EVENT_B = 0x02;
        }
    }

    impl EventTrait for MockEvt {
        type EventFlag = MockEvtFlag;

        fn matches_filter(&self, flag: Self::EventFlag) -> bool {
            flag.contains(self.as_flag())
        }
    }

    #[derive(Default, Clone, Debug, PartialEq)]
    struct MockAux;

    #[derive(Default, Clone, Debug, PartialEq)]
    struct MockDb;

    impl DatabaseTrait for MockDb {}

    #[derive(Clone, PartialEq, Debug)]
    struct MockSys<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
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
        phantom_c: PhantomData<H>,
    }

    impl<H, A, D, E> MockSys<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
        pub fn new(stage_filter: LoopStage, event_filter: E::EventFlag, error_out: bool) -> Self {
            MockSys {
                stage_filter: stage_filter,
                event_filter: event_filter,
                error_out: error_out,
                .. Default::default()
            }
        }
    }

    impl<H, A, D, E> From<(LoopStage, E::EventFlag, bool)> for MockSys<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
        fn from(value: (LoopStage, E::EventFlag, bool)) -> Self {
            MockSys::new(value.0, value.1, value.2)
        }
    }

    impl<H, A, D, E> Default for MockSys<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
        fn default() -> Self {
            MockSys {
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
                phantom_c: Default::default(),
            }
        }
    }

    impl<H, A, D, E> SystemTrait<H, A, D, E> for MockSys<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
        fn get_stage_filter(&self) -> LoopStage {
            self.stage_filter
        }
        fn get_event_filter(&self) -> E::EventFlag {
            self.event_filter
        }
        fn update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.update_arguments.push((*time, *delta_time));
            self.update_calls += 1;
            if self.error_out {
                Err(format_err!("MockSys.update() had an error"))
            } else {
                Ok(())
            }
        }
        fn dynamic_update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.dynamic_update_arguments.push((*time, *delta_time));
            self.dynamic_update_calls += 1;
            if self.error_out {
                Err(format_err!("MockSys.dynamic_update() had an error"))
            } else {
                Ok(())
            }
        }
        fn render(&mut self, _db: &D, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.render_arguments.push((*time, *delta_time));
            self.render_calls += 1;
            if self.error_out {
                Err(format_err!("MockSys.render() had an error"))
            } else {
                Ok(())
            }
        }
        fn handle_event(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, event: &E) -> Result<(), Error> {
            self.handle_event_arguments.push(event.clone());
            self.handle_event_calls += 1;
            if self.error_out {
                Err(format_err!("MockSys.handle_event() had an error"))
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct MockEvtMgr {
        events: VecDeque<MockEvt>,
        handle_events_calls: usize,
    }

    impl Default for MockEvtMgr {
        fn default() -> Self {
            MockEvtMgr {
                events: Default::default(),
                handle_events_calls: 0,
            }
        }
    }

    impl EventManagerTrait<MockEvt> for MockEvtMgr {
        fn dispatch_later(&mut self, event: MockEvt) {
            self.events.push_back(event)
        }
        fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
        where
            F: FnMut(&mut Self, &MockEvt) -> Result<bool, Error>,
        {
            self.handle_events_calls += 1;

            let tmp = self.events.iter().cloned().collect::<Vec<_>>();
            self.events.clear();

            for event in tmp {
                handler(self, &event)?;
            }

            Ok(true)
        }
    }

    fn create_populated_world() -> World<MockEvtMgr, MockAux, MockDb, MockEvt, MockSys<MockEvtMgr, MockAux, MockDb, MockEvt>> {
        let mut w: World<MockEvtMgr, MockAux, MockDb, MockEvt, MockSys<MockEvtMgr, MockAux, MockDb, MockEvt>> = World::default();

        w.systems = [
            MockSys::new(LoopStage::UPDATE, MockEvtFlag::empty(), false),
            MockSys::new(LoopStage::UPDATE, MockEvtFlag::empty(), false),
            MockSys::new(LoopStage::DYNAMIC_UPDATE, MockEvtFlag::empty(), false),
            MockSys::new(LoopStage::DYNAMIC_UPDATE, MockEvtFlag::empty(), false),
            MockSys::new(LoopStage::RENDER, MockEvtFlag::empty(), false),
            MockSys::new(LoopStage::RENDER, MockEvtFlag::empty(), false),
            MockSys::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_A, false),
            MockSys::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_B, false),
        ].iter().cloned().collect();

        w.event_manager.dispatch_later(MockEvt::TestEventA("lala-land".into()));
        w.event_manager.dispatch_later(MockEvt::TestEventB(100));

        w
    }

    #[test]
    fn add_system() {
        let mut w: World<MockEvtMgr, MockAux, MockDb, MockEvt, MockSys<MockEvtMgr, MockAux, MockDb, MockEvt>> = World::default();
        assert!(w.systems.is_empty());
        let sys = MockSys::new(LoopStage::UPDATE, MockEvtFlag::empty(), false);
        w.add_system(sys.clone());
        assert_eq!(w.systems.len(), 1);
        assert_eq!(w.systems.last().unwrap(), &sys);
        let into_sys = (LoopStage::UPDATE, MockEvtFlag::empty(), false);
        w.add_system(into_sys);
        assert_eq!(w.systems.len(), 2);
        assert_eq!(w.systems.last().unwrap(), &sys);
    }
    #[test]
    fn update_calls() {
        let mut w = create_populated_world();
        let r = w.update(&Duration::default(), &Duration::default());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::UPDATE) {
                assert_eq!(system.update_calls, 1);
            } else {
                assert_eq!(system.update_calls, 0);
            }
        }
    }
    #[test]
    fn update_arguments() {
        let mut w = create_populated_world();
        w.update(&Duration::new(1, 0), &Duration::new(0, 1)).unwrap();

        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::UPDATE) {
                assert!(system.update_arguments.len() > 0);
                for &(t, dt) in &system.update_arguments {
                    assert_eq!(t, Duration::new(1, 0));
                    assert_eq!(dt, Duration::new(0, 1));
                }
            }
        }
    }
    #[test]
    fn update_error() {
        let mut w: World<MockEvtMgr, MockAux, MockDb, MockEvt, MockSys<MockEvtMgr, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSys::new(LoopStage::UPDATE, MockEvtFlag::empty(), true));
        let r = w.update(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
    }
    #[test]
    fn dynamic_update_calls() {
        let mut w = create_populated_world();
        let r = w.dynamic_update(&Duration::default(), &Duration::default());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE) {
                assert_eq!(system.dynamic_update_calls, 1);
            } else {
                assert_eq!(system.dynamic_update_calls, 0);
            }
        }
    }
    #[test]
    fn dynamic_update_arguments() {
        let mut w = create_populated_world();
        w.dynamic_update(&Duration::new(1, 0), &Duration::new(0, 1)).unwrap();

        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::DYNAMIC_UPDATE) {
                assert!(system.dynamic_update_arguments.len() > 0);
                for &(t, dt) in &system.dynamic_update_arguments {
                    assert_eq!(t, Duration::new(1, 0));
                    assert_eq!(dt, Duration::new(0, 1));
                }
            }
        }
    }
    #[test]
    fn dynamic_update_error() {
        let mut w: World<MockEvtMgr, MockAux, MockDb, MockEvt, MockSys<MockEvtMgr, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSys::new(LoopStage::DYNAMIC_UPDATE, MockEvtFlag::empty(), true));
        let r = w.dynamic_update(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
    }
    #[test]
    fn render_calls() {
        let mut w = create_populated_world();
        let r = w.render(&Duration::default(), &Duration::default());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::RENDER) {
                assert_eq!(system.render_calls, 1);
            } else {
                assert_eq!(system.render_calls, 0);
            }
        }
    }
    #[test]
    fn render_arguments() {
        let mut w = create_populated_world();
        w.render(&Duration::new(1, 0), &Duration::new(0, 1)).unwrap();

        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::RENDER) {
                assert!(system.render_arguments.len() > 0);
                for &(t, dt) in &system.render_arguments {
                    assert_eq!(t, Duration::new(1, 0));
                    assert_eq!(dt, Duration::new(0, 1));
                }
            }
        }
    }
    #[test]
    fn render_error() {
        let mut w: World<MockEvtMgr, MockAux, MockDb, MockEvt, MockSys<MockEvtMgr, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSys::new(LoopStage::RENDER, MockEvtFlag::empty(), true));
        let r = w.render(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
    }
    #[test]
    fn handle_events_calls() {
        let mut w = create_populated_world();
        let r = w.handle_events();

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        assert_eq!(w.event_manager.handle_events_calls, 1);
        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) {
                if system.get_event_filter().contains(MockEvtFlag::TEST_EVENT_A) {
                    assert_eq!(system.handle_event_calls, 1);
                } else if system.get_event_filter().contains(MockEvtFlag::TEST_EVENT_B) {
                    assert_eq!(system.handle_event_calls, 1);
                } else {
                    assert_eq!(system.handle_event_calls, 0);
                }
            } else {
                assert_eq!(system.handle_event_calls, 0);
            }
        }
    }
    #[test]
    fn handle_events_arguments() {
        let mut w = create_populated_world();
        w.handle_events().unwrap();

        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) {
                if system.get_event_filter().contains(MockEvtFlag::TEST_EVENT_A) {
                    for event in &system.handle_event_arguments {
                        assert_eq!(event.as_flag(), MockEvtFlag::TEST_EVENT_A);
                    }
                } else if system.get_event_filter().contains(MockEvtFlag::TEST_EVENT_B) {
                    for event in &system.handle_event_arguments {
                        assert_eq!(event.as_flag(), MockEvtFlag::TEST_EVENT_B);
                    }
                } else {
                    assert!(system.handle_event_arguments.is_empty());
                }
            } else {
                assert!(system.handle_event_arguments.is_empty());
            }
        }
    }
    #[test]
    fn handle_events_error() {
        let mut w: World<MockEvtMgr, MockAux, MockDb, MockEvt, MockSys<MockEvtMgr, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSys::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_A, true));
        w.event_manager.dispatch_later(MockEvt::TestEventA("hello".into()));
        assert!(w.handle_events().is_err());
    }
}
