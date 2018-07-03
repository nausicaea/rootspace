use context::Context;
use event::{Event, EventFlag};
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use failure::Error;

#[derive(Default)]
pub struct EventCoordinator;

impl SystemTrait<Context, Event> for EventCoordinator {
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> EventFlag {
        EventFlag::SHUTDOWN | EventFlag::HARD_SHUTDOWN
    }

    // fn handle_event(&mut self, _ctx: &mut Context, event: &Event) -> Result<bool, Error> {
    //     if let &Event { flag: f, .. } = event {
    //         match f {
    //             EventFlag::SHUTDOWN =>
    //         }
    //     }
    // }
}
