use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use system::SystemTrait;
use loop_stage::LoopStage;
use event::{EventTrait, EventManagerTrait};
use database::DatabaseTrait;

pub trait WorldTrait {
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
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
    pub event_manager: H,
    pub auxiliary: A,
    pub database: D,
    phantom: PhantomData<E>,
    systems: Vec<S>,
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
            event_manager: Default::default(),
            auxiliary: Default::default(),
            database: Default::default(),
            phantom: Default::default(),
            systems: Default::default(),
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
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::FIXED_UPDATE) {
                system.fixed_update(&mut self.database, &mut self.event_manager, &mut self.auxiliary, time, delta_time)?;
            }
        }
        Ok(())
    }
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::UPDATE) {
                system.update(&mut self.database, &mut self.event_manager, &mut self.auxiliary, time, delta_time)?;
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
    use mock::{MockEvt, MockEvtFlag, MockEvtMgr, MockDb, MockAux, MockSysA};
    use super::*;

    fn create_populated_world() -> World<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>> {
        let mut w: World<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>> = World::default();

        w.systems = [
            MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::RENDER, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::RENDER, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_A, false),
            MockSysA::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_B, false),
        ].iter().cloned().collect();

        w.event_manager.dispatch_later(MockEvt::TestEventA("lala-land".into()));
        w.event_manager.dispatch_later(MockEvt::TestEventB(100));

        w
    }

    #[test]
    fn add_system() {
        let mut w: World<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>> = World::default();
        assert!(w.systems.is_empty());
        let sys = MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), false);
        w.add_system(sys.clone());
        assert_eq!(w.systems.len(), 1);
        assert_eq!(w.systems.last().unwrap(), &sys);
        let into_sys = (LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), false);
        w.add_system(into_sys);
        assert_eq!(w.systems.len(), 2);
        assert_eq!(w.systems.last().unwrap(), &sys);
    }
    #[test]
    fn fixed_update_calls() {
        let mut w = create_populated_world();
        let r = w.fixed_update(&Duration::default(), &Duration::default());

        assert!(r.is_ok(), "Got an unexpected error '{}'", r.unwrap_err());
        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::FIXED_UPDATE) {
                assert_eq!(system.fixed_update_calls, 1);
            } else {
                assert_eq!(system.fixed_update_calls, 0);
            }
        }
    }
    #[test]
    fn fixed_update_arguments() {
        let mut w = create_populated_world();
        w.fixed_update(&Duration::new(1, 0), &Duration::new(0, 1)).unwrap();

        for system in &w.systems {
            if system.get_stage_filter().contains(LoopStage::FIXED_UPDATE) {
                assert!(system.fixed_update_arguments.len() > 0);
                for &(t, dt) in &system.fixed_update_arguments {
                    assert_eq!(t, Duration::new(1, 0));
                    assert_eq!(dt, Duration::new(0, 1));
                }
            }
        }
    }
    #[test]
    fn fixed_update_error() {
        let mut w: World<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), true));
        let r = w.fixed_update(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
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
        let mut w: World<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSysA::new(LoopStage::UPDATE, MockEvtFlag::empty(), true));
        let r = w.update(&Duration::new(1, 0), &Duration::new(0, 1));
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
        let mut w: World<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSysA::new(LoopStage::RENDER, MockEvtFlag::empty(), true));
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
        let mut w: World<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt, MockSysA<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>> = World::default();
        w.systems.push(MockSysA::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_A, true));
        w.event_manager.dispatch_later(MockEvt::TestEventA("hello".into()));
        assert!(w.handle_events().is_err());
    }
}
