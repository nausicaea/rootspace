use std::time::Duration;
use failure::Error;
use loop_stage::LoopStage;
use event::EventTrait;

pub trait SystemTrait<C, E>
where
    E: EventTrait,
{
    fn get_stage_filter(&self) -> LoopStage;
    fn get_event_filter(&self) -> E::EventFlag {
        unimplemented!("Did you forget to implement the get_event_filter method?");
    }
    fn fixed_update(&mut self, _ctx: &mut C, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the fixed_update method?")
    }
    fn update(&mut self, _ctx: &mut C, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the update method?")
    }
    fn render(&mut self, _ctx: &mut C, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the render method?")
    }
    fn handle_event(&mut self, _ctx: &mut C, _e: &E) -> Result<(), Error> {
        unimplemented!("Did you forget to implement the handle_event method?")
    }
}
