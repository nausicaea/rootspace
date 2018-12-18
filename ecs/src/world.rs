use crate::event::{EventManagerTrait, EventTrait};
use failure::Error;
use crate::loop_stage::LoopStage;
use std::{marker::PhantomData, time::Duration};
use crate::system::SystemTrait;

/// A World must perform actions for four types of calls.
pub trait WorldTrait {
    /// The fixed update method is supposed to be called from the main loop at fixed time
    /// intervals.
    ///
    /// # Aruments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `fixed_update`.
    ///
    /// # Errors
    ///
    /// Will pass along any error encountered the respective systems.
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    /// The dynamic update method is supposed to be called from the main loop just before the
    /// render call.
    ///
    /// # Aruments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `update`.
    ///
    /// # Errors
    ///
    /// Will pass along any error encountered the respective systems.
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    /// The render method is supposed to be called when a re-draw of the graphical representation
    /// is desired.
    ///
    /// # Aruments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `render`.
    ///
    /// # Errors
    ///
    /// Will pass along any error encountered the respective systems.
    fn render(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error>;
    /// The handle events method is supposed to be called when pending events or messages should be
    /// handled by the connected systems. If this method returns `Ok(true)`, the execution of the
    /// main loop shall continue, otherwise it shall abort.
    ///
    /// # Errors
    ///
    /// Will pass along any error encountered the respective systems.
    fn handle_events(&mut self) -> Result<bool, Error>;
}

/// This is the default implementation of the `WorldTrait` provided by this library.
pub struct World<E, C, S> {
    /// The context must be capable of managing events and messages. Additionally, any behaviour
    /// may be added.
    pub context: C,
    /// Holds all systems at use in this `World`.
    systems: Vec<S>,
    _e: PhantomData<E>,
}

impl<E, C, S> World<E, C, S>
where
    E: EventTrait,
    C: Default + EventManagerTrait<E>,
    S: SystemTrait<C, E>,
{
    /// Adds a new system to the `World`.
    ///
    /// # Arguments
    ///
    /// * `system` - A type that must implement the `Into<S>` trait, where `S: SystemTrait<_, _>`.
    pub fn add_system<T: Into<S>>(&mut self, system: T) {
        self.systems.push(system.into());
    }
}

impl<E, C, S> Default for World<E, C, S>
where
    E: EventTrait,
    C: Default + EventManagerTrait<E>,
    S: SystemTrait<C, E>,
{
    fn default() -> Self {
        World {
            context: Default::default(),
            systems: Vec::default(),
            _e: PhantomData::default(),
        }
    }
}

impl<E, C, S> WorldTrait for World<E, C, S>
where
    E: EventTrait,
    C: Default + EventManagerTrait<E>,
    S: SystemTrait<C, E>,
{
    fn fixed_update(&mut self, t: &Duration, dt: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::FIXED_UPDATE) {
                system.fixed_update(&mut self.context, t, dt)?;
            }
        }
        Ok(())
    }
    fn update(&mut self, t: &Duration, dt: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::UPDATE) {
                system.update(&mut self.context, t, dt)?;
            }
        }
        Ok(())
    }
    fn render(&mut self, t: &Duration, dt: &Duration) -> Result<(), Error> {
        for system in &mut self.systems {
            if system.get_stage_filter().contains(LoopStage::RENDER) {
                system.render(&mut self.context, t, dt)?;
            }
        }
        Ok(())
    }
    fn handle_events(&mut self) -> Result<bool, Error> {
        let systems = &mut self.systems;

        self.context.handle_events(|ctx, event| {
            let mut statuses: Vec<bool> = Vec::with_capacity(systems.len());

            for system in systems.iter_mut() {
                if system.get_stage_filter().contains(LoopStage::HANDLE_EVENTS)
                    && event.matches_filter(system.get_event_filter())
                {
                    statuses.push(system.handle_event(ctx, event)?);
                }
            }

            Ok(statuses.iter().all(|s| *s))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockCtx, MockEvt, MockEvtFlag, MockSysA};

    fn create_populated_world() -> World<MockEvt, MockCtx<MockEvt>, MockSysA<MockCtx<MockEvt>, MockEvt>> {
        let mut w: World<MockEvt, MockCtx<MockEvt>, MockSysA<MockCtx<MockEvt>, MockEvt>> = Default::default();

        w.systems = vec![
            MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::UPDATE, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::RENDER, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::RENDER, MockEvtFlag::empty(), false),
            MockSysA::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_A, false),
            MockSysA::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_B, false),
        ];

        w.context.dispatch_later(MockEvt::TestEventA("lala-land".into()));
        w.context.dispatch_later(MockEvt::TestEventB(100));

        w
    }

    #[test]
    fn add_system() {
        let mut w: World<MockEvt, MockCtx<MockEvt>, MockSysA<MockCtx<MockEvt>, MockEvt>> = World::default();
        assert!(w.systems.is_empty());

        let sys = MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), false);
        w.add_system(sys.clone());
        assert_eq!(w.systems.len(), 1);

        let into_sys = (LoopStage::UPDATE, MockEvtFlag::empty(), false);
        w.add_system(into_sys);
        assert_eq!(w.systems.len(), 2);
        assert_eq!(w.systems.last().unwrap().get_stage_filter(), LoopStage::UPDATE);
    }
    #[test]
    fn fixed_update_calls() {
        let mut w = create_populated_world();
        let r = w.fixed_update(&Duration::default(), &Duration::default());

        assert!(r.is_ok());
        for system in &w.systems {
            assert_eq!(system.stage_filter_calls(), 1);
            assert_eq!(system.event_filter_calls(), 0);
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
        let mut w: World<MockEvt, MockCtx<MockEvt>, MockSysA<MockCtx<MockEvt>, MockEvt>> = World::default();
        w.systems
            .push(MockSysA::new(LoopStage::FIXED_UPDATE, MockEvtFlag::empty(), true));
        let r = w.fixed_update(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
    }
    #[test]
    fn update_calls() {
        let mut w = create_populated_world();
        let r = w.update(&Duration::default(), &Duration::default());

        assert!(r.is_ok());
        for system in &w.systems {
            assert_eq!(system.stage_filter_calls(), 1);
            assert_eq!(system.event_filter_calls(), 0);
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
        let mut w: World<MockEvt, MockCtx<MockEvt>, MockSysA<MockCtx<MockEvt>, MockEvt>> = World::default();
        w.systems
            .push(MockSysA::new(LoopStage::UPDATE, MockEvtFlag::empty(), true));
        let r = w.update(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
    }
    #[test]
    fn render_calls() {
        let mut w = create_populated_world();
        let r = w.render(&Duration::default(), &Duration::default());

        assert!(r.is_ok());
        for system in &w.systems {
            assert_eq!(system.stage_filter_calls(), 1);
            assert_eq!(system.event_filter_calls(), 0);
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
        let mut w: World<MockEvt, MockCtx<MockEvt>, MockSysA<MockCtx<MockEvt>, MockEvt>> = World::default();
        w.systems
            .push(MockSysA::new(LoopStage::RENDER, MockEvtFlag::empty(), true));
        let r = w.render(&Duration::new(1, 0), &Duration::new(0, 1));
        assert!(r.is_err());
    }
    #[test]
    fn handle_events_calls() {
        let mut w = create_populated_world();
        let r = w.handle_events();

        assert!(r.is_ok());
        assert_eq!(w.context.handle_events_calls, 1);
        for system in &w.systems {
            assert_eq!(system.stage_filter_calls(), 2);
            if system.get_stage_filter().contains(LoopStage::HANDLE_EVENTS) {
                assert_eq!(system.event_filter_calls(), 2);
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
        let mut w: World<MockEvt, MockCtx<MockEvt>, MockSysA<MockCtx<MockEvt>, MockEvt>> = World::default();
        w.systems
            .push(MockSysA::new(LoopStage::HANDLE_EVENTS, MockEvtFlag::TEST_EVENT_A, true));
        w.context.dispatch_later(MockEvt::TestEventA("hello".into()));
        assert!(w.handle_events().is_err());
    }
}
