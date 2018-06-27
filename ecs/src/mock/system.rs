use event::EventTrait;
use failure::Error;
use loop_stage::LoopStage;
use std::marker::PhantomData;
use std::sync::RwLock;
use std::time::Duration;
use system::SystemTrait;

#[derive(Debug)]
pub struct MockSysA<C, E>
where
    E: EventTrait,
{
    pub stage_filter: LoopStage,
    pub sfc: RwLock<usize>,
    pub event_filter: E::EventFlag,
    pub efc: RwLock<usize>,
    pub error_out: bool,
    pub fixed_update_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_calls: usize,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_calls: usize,
    pub render_arguments: Vec<(Duration, Duration)>,
    pub handle_event_calls: usize,
    pub handle_event_arguments: Vec<E>,
    phantom_c: PhantomData<C>,
}

impl<C, E> MockSysA<C, E>
where
    E: EventTrait,
{
    pub fn new(stage_filter: LoopStage, event_filter: E::EventFlag, error_out: bool) -> Self {
        MockSysA {
            stage_filter,
            event_filter,
            error_out,
            ..MockSysA::default()
        }
    }

    pub fn stage_filter_calls(&self) -> usize {
        *self.sfc.read().unwrap()
    }

    pub fn event_filter_calls(&self) -> usize {
        *self.efc.read().unwrap()
    }
}

impl<C, E> Clone for MockSysA<C, E>
where
    E: EventTrait,
{
    fn clone(&self) -> MockSysA<C, E> {
        MockSysA {
            stage_filter: self.stage_filter,
            sfc: RwLock::new(self.stage_filter_calls()),
            event_filter: self.event_filter,
            efc: RwLock::new(self.event_filter_calls()),
            error_out: self.error_out,
            fixed_update_calls: self.fixed_update_calls,
            fixed_update_arguments: self.fixed_update_arguments.clone(),
            update_calls: self.update_calls,
            update_arguments: self.update_arguments.clone(),
            render_calls: self.render_calls,
            render_arguments: self.render_arguments.clone(),
            handle_event_calls: self.handle_event_calls,
            handle_event_arguments: self.handle_event_arguments.clone(),
            phantom_c: PhantomData::default(),
        }
    }
}

impl<C, E> Default for MockSysA<C, E>
where
    E: EventTrait,
{
    fn default() -> Self {
        MockSysA {
            stage_filter: LoopStage::default(),
            sfc: RwLock::default(),
            event_filter: Default::default(),
            efc: RwLock::default(),
            error_out: Default::default(),
            fixed_update_calls: 0,
            fixed_update_arguments: Vec::new(),
            update_calls: 0,
            update_arguments: Vec::new(),
            render_calls: 0,
            render_arguments: Vec::new(),
            handle_event_calls: 0,
            handle_event_arguments: Vec::new(),
            phantom_c: PhantomData::default(),
        }
    }
}

impl<C, E> From<(LoopStage, E::EventFlag, bool)> for MockSysA<C, E>
where
    E: EventTrait,
{
    fn from(value: (LoopStage, E::EventFlag, bool)) -> Self {
        MockSysA::new(value.0, value.1, value.2)
    }
}

impl<C, E> SystemTrait<C, E> for MockSysA<C, E>
where
    E: EventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        let mut calls = self.sfc.write().unwrap();
        *calls += 1;
        self.stage_filter
    }
    fn get_event_filter(&self) -> E::EventFlag {
        let mut calls = self.efc.write().unwrap();
        *calls += 1;
        self.event_filter
    }
    fn fixed_update(
        &mut self,
        _ctx: &mut C,
        time: &Duration,
        delta_time: &Duration,
    ) -> Result<(), Error> {
        self.fixed_update_arguments.push((*time, *delta_time));
        self.fixed_update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn update(
        &mut self,
        _ctx: &mut C,
        time: &Duration,
        delta_time: &Duration,
    ) -> Result<(), Error> {
        self.update_arguments.push((*time, *delta_time));
        self.update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn render(
        &mut self,
        _ctx: &mut C,
        time: &Duration,
        delta_time: &Duration,
    ) -> Result<(), Error> {
        self.render_arguments.push((*time, *delta_time));
        self.render_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.render() had an error"))
        } else {
            Ok(())
        }
    }
    fn handle_event(&mut self, _ctx: &mut C, event: &E) -> Result<(), Error> {
        self.handle_event_arguments.push(event.clone());
        self.handle_event_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.handle_event() had an error"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct MockSysB<C, E>
where
    E: EventTrait,
{
    pub stage_filter: LoopStage,
    pub sfc: RwLock<usize>,
    pub event_filter: E::EventFlag,
    pub efc: RwLock<usize>,
    pub error_out: bool,
    pub fixed_update_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_calls: usize,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_calls: usize,
    pub render_arguments: Vec<(Duration, Duration)>,
    pub handle_event_calls: usize,
    pub handle_event_arguments: Vec<E>,
    phantom_c: PhantomData<C>,
}

impl<C, E> MockSysB<C, E>
where
    E: EventTrait,
{
    pub fn new(stage_filter: LoopStage, event_filter: E::EventFlag, error_out: bool) -> Self {
        MockSysB {
            stage_filter,
            event_filter,
            error_out,
            ..MockSysB::default()
        }
    }

    pub fn stage_filter_calls(&self) -> usize {
        *self.sfc.read().unwrap()
    }

    pub fn event_filter_calls(&self) -> usize {
        *self.efc.read().unwrap()
    }
}

impl<C, E> Clone for MockSysB<C, E>
where
    E: EventTrait,
{
    fn clone(&self) -> MockSysB<C, E> {
        MockSysB {
            stage_filter: self.stage_filter,
            sfc: RwLock::new(self.stage_filter_calls()),
            event_filter: self.event_filter,
            efc: RwLock::new(self.event_filter_calls()),
            error_out: self.error_out,
            fixed_update_calls: self.fixed_update_calls,
            fixed_update_arguments: self.fixed_update_arguments.clone(),
            update_calls: self.update_calls,
            update_arguments: self.update_arguments.clone(),
            render_calls: self.render_calls,
            render_arguments: self.render_arguments.clone(),
            handle_event_calls: self.handle_event_calls,
            handle_event_arguments: self.handle_event_arguments.clone(),
            phantom_c: PhantomData::default(),
        }
    }
}

impl<C, E> Default for MockSysB<C, E>
where
    E: EventTrait,
{
    fn default() -> Self {
        MockSysB {
            stage_filter: LoopStage::default(),
            sfc: RwLock::default(),
            event_filter: Default::default(),
            efc: RwLock::default(),
            error_out: Default::default(),
            fixed_update_calls: 0,
            fixed_update_arguments: Vec::new(),
            update_calls: 0,
            update_arguments: Vec::new(),
            render_calls: 0,
            render_arguments: Vec::new(),
            handle_event_calls: 0,
            handle_event_arguments: Vec::new(),
            phantom_c: PhantomData::default(),
        }
    }
}

impl<C, E> From<(LoopStage, E::EventFlag, bool)> for MockSysB<C, E>
where
    E: EventTrait,
{
    fn from(value: (LoopStage, E::EventFlag, bool)) -> Self {
        MockSysB::new(value.0, value.1, value.2)
    }
}

impl<C, E> SystemTrait<C, E> for MockSysB<C, E>
where
    E: EventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        let mut calls = self.sfc.write().unwrap();
        *calls += 1;
        self.stage_filter
    }
    fn get_event_filter(&self) -> E::EventFlag {
        let mut calls = self.efc.write().unwrap();
        *calls += 1;
        self.event_filter
    }
    fn fixed_update(
        &mut self,
        _ctx: &mut C,
        time: &Duration,
        delta_time: &Duration,
    ) -> Result<(), Error> {
        self.fixed_update_arguments.push((*time, *delta_time));
        self.fixed_update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn update(
        &mut self,
        _ctx: &mut C,
        time: &Duration,
        delta_time: &Duration,
    ) -> Result<(), Error> {
        self.update_arguments.push((*time, *delta_time));
        self.update_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.update() had an error"))
        } else {
            Ok(())
        }
    }
    fn render(
        &mut self,
        _ctx: &mut C,
        time: &Duration,
        delta_time: &Duration,
    ) -> Result<(), Error> {
        self.render_arguments.push((*time, *delta_time));
        self.render_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.render() had an error"))
        } else {
            Ok(())
        }
    }
    fn handle_event(&mut self, _ctx: &mut C, event: &E) -> Result<(), Error> {
        self.handle_event_arguments.push(event.clone());
        self.handle_event_calls += 1;
        if self.error_out {
            Err(format_err!("MockSysA.handle_event() had an error"))
        } else {
            Ok(())
        }
    }
}
