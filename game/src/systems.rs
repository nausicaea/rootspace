use std::time::Duration;
use failure::Error;
use ecs::system::SystemTrait;
use ecs::loop_stage::LoopStage;
use ecs::database::DatabaseTrait;
use ecs::event::{EventTrait, EventManagerTrait};

pub enum SystemGroup {
}

impl<H, A, D, E> SystemTrait<H, A, D, E> for SystemGroup where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::all()
    }
    fn get_event_filter(&self) -> E::EventFlag {
        Default::default()
    }
    fn update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        Ok(())
    }
    fn dynamic_update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        Ok(())
    }
    fn render(&mut self, _db: &D, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        Ok(())
    }
    fn handle_event(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, _event: &E) -> Result<(), Error> {
        Ok(())
    }
}

