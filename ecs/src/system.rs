use std::time::Duration;
use failure::Error;
use loop_stage::LoopStage;
use database::DatabaseTrait;
use event::EventTrait;

pub trait SystemTrait<A, D, E> where E: EventTrait, D: DatabaseTrait {
    fn get_stage_filter(&self) -> LoopStage;
    fn get_event_filter(&self) -> E::EventFlag;
    fn update(&mut self, _db: &mut D, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the update method?")
    }
    fn dynamic_update(&mut self, _db: &mut D, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the dynamic update method?")
    }
    fn render(&mut self, _db: &D, _aux: &mut A, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the render method?")
    }
    fn handle_event(&mut self, _db: &mut D, _aux: &mut A, _event: &E) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the handle_event method?")
    }
}

