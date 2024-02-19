use ecs::Resources;
use serde::{Deserialize, Serialize};

use super::CommandTrait;
use crate::resources::Statistics;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerfCommand;

impl CommandTrait for PerfCommand {
    fn name(&self) -> &'static str {
        "perf"
    }

    fn description(&self) -> &'static str {
        "Provides access to runtime statistics about the engine"
    }

    fn run(&self, res: &Resources, _args: &[String]) -> anyhow::Result<()> {
        let stats = res.borrow::<Statistics>();

        let avg_world_dc = stats.average_world_draw_calls();
        let avg_ui_dc = stats.average_ui_draw_calls();
        let avg_ft = stats.average_frame_time();
        let avg_lt = stats.average_loop_time();

        println!("Average number of world space draw calls: {}", avg_world_dc);
        println!("Average number of UI draw calls: {}", avg_ui_dc);
        println!("Average time spent rendering: {:?}", avg_ft);
        println!("Average time each main loop iteration: {:?}", avg_lt);

        Ok(())
    }
}
