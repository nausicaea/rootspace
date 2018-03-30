use std::time::Duration;
use failure::Error;
use loop_stage::LoopStage;
use database::DatabaseTrait;
use event::{EventTrait, EventManagerTrait};

pub trait SystemTrait<H, A, D, E> where H: EventManagerTrait<E>, E: EventTrait, D: DatabaseTrait {
    fn get_stage_filter(&self) -> LoopStage;
    fn get_event_filter(&self) -> E::EventFlag;
    fn fixed_update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the update method?")
    }
    fn update(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the dynamic update method?")
    }
    fn render(&mut self, _db: &D, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the render method?")
    }
    fn handle_event(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, _event: &E) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the handle_event method?")
    }
}
