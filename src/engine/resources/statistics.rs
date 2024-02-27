use std::{collections::VecDeque, time::Duration};

use crate::ecs::resource::Resource;
use crate::ecs::with_dependencies::WithDependencies;
use serde::{Deserialize, Serialize};

const DRAW_CALL_WINDOW: usize = 10;
const FRAME_TIME_WINDOW: usize = 10;
const LOOP_TIME_WINDOW: usize = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    #[serde(skip)]
    draw_calls: VecDeque<(usize, usize)>,
    #[serde(skip)]
    frame_times: VecDeque<Duration>,
    #[serde(skip)]
    loop_times: VecDeque<Duration>,
}

impl Statistics {
    pub fn average_world_draw_calls(&self) -> f32 {
        self.draw_calls.iter().map(|(wdc, _)| wdc).sum::<usize>() as f32 / DRAW_CALL_WINDOW as f32
    }

    pub fn average_ui_draw_calls(&self) -> f32 {
        self.draw_calls.iter().map(|(_, udc)| udc).sum::<usize>() as f32 / DRAW_CALL_WINDOW as f32
    }

    pub fn average_frame_time(&self) -> Duration {
        self.frame_times.iter().sum::<Duration>() / FRAME_TIME_WINDOW as u32
    }

    pub fn average_loop_time(&self) -> Duration {
        self.loop_times.iter().sum::<Duration>() / LOOP_TIME_WINDOW as u32
    }

    pub fn update_draw_calls(&mut self, world_draw_calls: usize, ui_draw_calls: usize) {
        self.draw_calls.push_front((world_draw_calls, ui_draw_calls));
        if self.draw_calls.len() > DRAW_CALL_WINDOW {
            self.draw_calls.truncate(DRAW_CALL_WINDOW);
        }
    }

    pub fn update_frame_time(&mut self, frame_time: Duration) {
        self.frame_times.push_front(frame_time);
        if self.frame_times.len() > FRAME_TIME_WINDOW {
            self.frame_times.truncate(FRAME_TIME_WINDOW);
        }
    }

    pub fn update_loop_times(&mut self, loop_duration: Duration) {
        self.loop_times.push_front(loop_duration);
        if self.loop_times.len() > LOOP_TIME_WINDOW {
            self.loop_times.truncate(LOOP_TIME_WINDOW);
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            draw_calls: VecDeque::with_capacity(DRAW_CALL_WINDOW),
            frame_times: VecDeque::with_capacity(FRAME_TIME_WINDOW),
            loop_times: VecDeque::with_capacity(LOOP_TIME_WINDOW),
        }
    }
}

impl Resource for Statistics {}

impl<D> WithDependencies<D> for Statistics {
    async fn with_deps(_: &D) -> Result<Self, anyhow::Error> {
        Ok(Statistics::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::registry::{End, ResourceRegistry};
    use crate::ecs::world::World;
    use crate::Reg;

    use super::*;

    #[test]
    fn statistics_reg_macro() {
        type _RR = Reg![Statistics];
    }

    #[test]
    fn statistics_resource_registry() {
        let _rr = ResourceRegistry::push(End, Statistics::default());
    }

    #[test]
    fn statistics_world() {
        let _w = World::with_dependencies::<Reg![Statistics], Reg![], Reg![], Reg![], _>(&()).unwrap();
    }
}
