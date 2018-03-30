use std::collections::VecDeque;
use std::marker::PhantomData;
use std::time::Duration;
use std::thread;
use failure::Error;
use database::DatabaseTrait;
use event::{EventTrait, EventManagerTrait};
use loop_stage::LoopStage;
use system::SystemTrait;
use world::WorldTrait;

#[derive(Clone, Debug, PartialEq)]
pub enum MockEvt {
    TestEventA(String),
    TestEventB(u32),
}

impl MockEvt {
    pub fn as_flag(&self) -> MockEvtFlag {
        match *self {
            MockEvt::TestEventA(_) => MockEvtFlag::TEST_EVENT_A,
            MockEvt::TestEventB(_) => MockEvtFlag::TEST_EVENT_B,
        }
    }
}

bitflags! {
    pub struct MockEvtFlag: u8 {
        const TEST_EVENT_A = 0x01;
        const TEST_EVENT_B = 0x02;
    }
}

impl Default for MockEvtFlag {
    fn default() -> Self {
        MockEvtFlag::all()
    }
}

impl EventTrait for MockEvt {
    type EventFlag = MockEvtFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.as_flag())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MockEvtMgr<E>
where
    E: EventTrait,
{
    pub events: VecDeque<E>,
    pub handle_events_calls: usize,
}

impl<E> Default for MockEvtMgr<E>
where
    E: EventTrait,
{
    fn default() -> Self {
        MockEvtMgr {
            events: Default::default(),
            handle_events_calls: 0,
        }
    }
}

impl<E> EventManagerTrait<E> for MockEvtMgr<E>
where
    E: EventTrait,
{
    fn dispatch_later(&mut self, event: E) {
        self.events.push_back(event)
    }
    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
        where
            F: FnMut(&mut Self, &E) -> Result<bool, Error>,
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

#[derive(Default, Clone, Debug, PartialEq)]
pub struct MockAux;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct MockDb;

impl DatabaseTrait for MockDb {}

#[derive(Clone, PartialEq, Debug)]
pub struct MockSysA<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    pub stage_filter: LoopStage,
    pub event_filter: E::EventFlag,
    pub error_out: bool,
    pub fixed_update_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_calls: usize,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_calls: usize,
    pub render_arguments: Vec<(Duration, Duration)>,
    pub handle_event_calls: usize,
    pub handle_event_arguments: Vec<E>,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<D>,
    phantom_c: PhantomData<H>,
}

impl<H, A, D, E> MockSysA<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    pub fn new(stage_filter: LoopStage, event_filter: E::EventFlag, error_out: bool) -> Self {
        MockSysA {
            stage_filter: stage_filter,
            event_filter: event_filter,
            error_out: error_out,
            .. Default::default()
        }
    }
}

impl<H, A, D, E> From<(LoopStage, E::EventFlag, bool)> for MockSysA<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn from(value: (LoopStage, E::EventFlag, bool)) -> Self {
        MockSysA::new(value.0, value.1, value.2)
    }
}

impl<H, A, D, E> Default for MockSysA<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn default() -> Self {
        MockSysA {
            stage_filter: Default::default(),
            event_filter: Default::default(),
            error_out: Default::default(),
            fixed_update_calls: 0,
            fixed_update_arguments: Vec::new(),
            update_calls: 0,
            update_arguments: Vec::new(),
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

impl<H, A, D, E> SystemTrait<H, A, D, E> for MockSysA<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn get_stage_filter(&self) -> LoopStage {
        self.stage_filter
    }
    fn get_event_filter(&self) -> E::EventFlag {
        self.event_filter
    }
    fn fixed_update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.fixed_update_arguments.push((*time, *delta_time));
        self.fixed_update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.update_arguments.push((*time, *delta_time));
        self.update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn render(&mut self, _db: &D, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.render_arguments.push((*time, *delta_time));
        self.render_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.render() had an error"))
        } else {
            Ok(())
        }
    }
    fn handle_event(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, event: &E) -> Result<(), Error> {
        self.handle_event_arguments.push(event.clone());
        self.handle_event_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.handle_event() had an error"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct MockSysB<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    pub stage_filter: LoopStage,
    pub event_filter: E::EventFlag,
    pub error_out: bool,
    pub fixed_update_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_calls: usize,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_calls: usize,
    pub render_arguments: Vec<(Duration, Duration)>,
    pub handle_event_calls: usize,
    pub handle_event_arguments: Vec<E>,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<D>,
    phantom_c: PhantomData<H>,
}

impl<H, A, D, E> MockSysB<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    pub fn new(stage_filter: LoopStage, event_filter: E::EventFlag, error_out: bool) -> Self {
        MockSysB {
            stage_filter: stage_filter,
            event_filter: event_filter,
            error_out: error_out,
            .. Default::default()
        }
    }
}

impl<H, A, D, E> From<(LoopStage, E::EventFlag, bool)> for MockSysB<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn from(value: (LoopStage, E::EventFlag, bool)) -> Self {
        MockSysB::new(value.0, value.1, value.2)
    }
}

impl<H, A, D, E> Default for MockSysB<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn default() -> Self {
        MockSysB {
            stage_filter: Default::default(),
            event_filter: Default::default(),
            error_out: Default::default(),
            fixed_update_calls: 0,
            fixed_update_arguments: Vec::new(),
            update_calls: 0,
            update_arguments: Vec::new(),
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

impl<H, A, D, E> SystemTrait<H, A, D, E> for MockSysB<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn get_stage_filter(&self) -> LoopStage {
        self.stage_filter
    }
    fn get_event_filter(&self) -> E::EventFlag {
        self.event_filter
    }
    fn fixed_update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.fixed_update_arguments.push((*time, *delta_time));
        self.fixed_update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysB.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.update_arguments.push((*time, *delta_time));
        self.update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysB.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn render(&mut self, _db: &D, _aux: &mut A, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.render_arguments.push((*time, *delta_time));
        self.render_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysB.render() had an error"))
        } else {
            Ok(())
        }
    }
    fn handle_event(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, event: &E) -> Result<(), Error> {
        self.handle_event_arguments.push(event.clone());
        self.handle_event_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysB.handle_event() had an error"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct MockWorld {
    pub max_iterations: usize,
    pub render_duration: Option<Duration>,
    pub fixed_update_error_out: bool,
    pub fixed_update_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_error_out: bool,
    pub update_calls: usize,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_error_out: bool,
    pub render_calls: usize,
    pub render_arguments: Vec<(Duration, Duration)>,
    pub handle_events_error_out: bool,
    pub handle_events_calls: usize,
}

impl Default for MockWorld {
    fn default() -> Self {
        MockWorld {
            max_iterations: 10,
            render_duration: None,
            fixed_update_error_out: false,
            fixed_update_calls: 0,
            fixed_update_arguments: Vec::new(),
            update_error_out: false,
            update_calls: 0,
            update_arguments: Vec::new(),
            render_error_out: false,
            render_calls: 0,
            render_arguments: Vec::new(),
            handle_events_error_out: false,
            handle_events_calls: 0,
        }
    }
}

impl WorldTrait for MockWorld {
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.fixed_update_arguments.push((*time, *delta_time));
        self.fixed_update_calls += 1;
        if self.fixed_update_error_out {
            Err(format_err!("MockWorld.update() had an error."))
        } else {
            Ok(())
        }
    }
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.update_arguments.push((*time, *delta_time));
        self.update_calls += 1;
        if self.update_error_out {
            Err(format_err!("MockWorld.update() had an error."))
        } else {
            Ok(())
        }
    }
    fn render(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.render_arguments.push((*time, *delta_time));
        self.render_calls += 1;

        if let Some(d) = self.render_duration {
            thread::sleep(d);
        }
        if self.render_error_out {
            Err(format_err!("MockWorld.render() had an error."))
        } else {
            Ok(())
        }
    }
    fn handle_events(&mut self) -> Result<bool, Error> {
        self.handle_events_calls += 1;
        if self.handle_events_error_out {
            Err(format_err!("MockWorld.handle_events() had an error."))
        } else {
            Ok(self.handle_events_calls < self.max_iterations)
        }
    }
}
