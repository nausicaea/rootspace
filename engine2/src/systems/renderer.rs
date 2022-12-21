use ecs::{System, SerializationName};

use crate::resources::statistics::Statistics;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Renderer;

impl SerializationName for Renderer {}

impl System for Renderer {
    fn run(&mut self, res: &ecs::Resources, t: &std::time::Duration, dt: &std::time::Duration) {
        let start_mark = std::time::Instant::now();
        let mut world_draw_calls: usize = 0;
        let mut ui_draw_calls: usize = 0;

        let mut stats = res.borrow_mut::<Statistics>();
        stats.update_draw_calls(world_draw_calls, ui_draw_calls);
        stats.update_frame_time(start_mark.elapsed());
    }
}
