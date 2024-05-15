use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::ecs::{resource::Resource, with_dependencies::WithDependencies};

const WINDOW_SIZE: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    draw_calls: VecDeque<usize>,
    render_durations: VecDeque<Duration>,
    render_prepare_durations: VecDeque<Duration>,
    render_draw_durations: VecDeque<Duration>,
    redraw_intervals: VecDeque<Duration>,
    maintenance_intervals: VecDeque<Duration>,
}

impl Statistics {
    pub fn mean_draw_calls(&self) -> f32 {
        self.draw_calls.iter().sum::<usize>() as f32 / WINDOW_SIZE as f32
    }

    pub fn mean_render_duration(&self) -> Duration {
        self.render_durations.iter().sum::<Duration>().div_f32(WINDOW_SIZE as f32)
    }

    pub fn mean_render_prepare_duration(&self) -> Duration {
        self.render_prepare_durations.iter().sum::<Duration>().div_f32(WINDOW_SIZE as f32)
    }

    pub fn mean_render_draw_duration(&self) -> Duration {
        self.render_draw_durations.iter().sum::<Duration>().div_f32(WINDOW_SIZE as f32)
    }

    pub fn mean_redraw_interval(&self) -> Duration {
        self.redraw_intervals.iter().sum::<Duration>().div_f32(WINDOW_SIZE as f32)
    }

    pub fn mean_maintenance_interval(&self) -> Duration {
        self.maintenance_intervals.iter().sum::<Duration>().div_f32(WINDOW_SIZE as f32)
    }

    pub fn update_render_stats(&mut self,
       draw_calls: usize,
       frame_duration: Duration,
       prepare_duration: Duration,
       draw_duration: Duration,
    ) {
        self.draw_calls.push_front(draw_calls);
        self.render_durations.push_front(frame_duration);
        self.render_prepare_durations.push_front(prepare_duration);
        self.render_draw_durations.push_front(draw_duration);

        if self.draw_calls.len() > WINDOW_SIZE {
            self.draw_calls.truncate(WINDOW_SIZE);
        }
        if self.render_durations.len() > WINDOW_SIZE {
            self.render_durations.truncate(WINDOW_SIZE);
        }
        if self.render_prepare_durations.len() > WINDOW_SIZE {
            self.render_prepare_durations.truncate(WINDOW_SIZE);
        }
        if self.render_draw_durations.len() > WINDOW_SIZE {
            self.render_draw_durations.truncate(WINDOW_SIZE);
        }
    }

    pub fn update_redraw_intervals(&mut self, redraw_interval: Duration) {
        self.redraw_intervals.push_front(redraw_interval);
        if self.redraw_intervals.len() > WINDOW_SIZE {
            self.redraw_intervals.truncate(WINDOW_SIZE);
        }
    }

    pub fn update_maintenance_intervals(&mut self, maintenance_interval: Duration) {
        self.maintenance_intervals.push_front(maintenance_interval);
        if self.maintenance_intervals.len() > WINDOW_SIZE {
            self.maintenance_intervals.truncate(WINDOW_SIZE);
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            draw_calls: VecDeque::with_capacity(WINDOW_SIZE),
            render_durations: VecDeque::with_capacity(WINDOW_SIZE),
            render_prepare_durations: VecDeque::with_capacity(WINDOW_SIZE),
            render_draw_durations: VecDeque::with_capacity(WINDOW_SIZE),
            redraw_intervals: VecDeque::with_capacity(WINDOW_SIZE),
            maintenance_intervals: VecDeque::with_capacity(WINDOW_SIZE),
        }
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Loop and Render Stats:\nDraw calls (mean): {}\nRender duration (mean): {}\nPrepare duration (mean): {}\nDraw duration (mean): {}\nRedraw interval (mean): {}\nMaintenance interval (mean): {}\n",
               self.mean_draw_calls(),
               humantime::format_duration(self.mean_render_duration()),
               humantime::format_duration(self.mean_render_prepare_duration()),
               humantime::format_duration(self.mean_render_draw_duration()),
               humantime::format_duration(self.mean_redraw_interval()),
               humantime::format_duration(self.mean_maintenance_interval()),
        )
    }
}

impl Resource for Statistics {}

impl<D> WithDependencies<D> for Statistics {
    #[tracing::instrument(skip_all)]
    async fn with_deps(_: &D) -> Result<Self, anyhow::Error> {
        Ok(Statistics::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ecs::{
            registry::{End, ResourceRegistry},
            world::World,
        },
        Reg,
    };

    #[test]
    fn statistics_reg_macro() {
        type _RR = Reg![Statistics];
    }

    #[test]
    fn statistics_resource_registry() {
        let _rr = ResourceRegistry::push(End, Statistics::default());
    }

    #[tokio::test]
    async fn statistics_world() {
        let _w = World::with_dependencies::<Reg![Statistics], Reg![], Reg![], (), Reg![], _>(&())
            .await
            .unwrap();
    }
}
