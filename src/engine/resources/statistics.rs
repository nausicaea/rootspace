use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::ecs::{resource::Resource, with_dependencies::WithDependencies};

const DRAW_CALL_WINDOW: usize = 10;
const RENDER_DURATION_WINDOW: usize = 10;
const REDRAW_INTERVAL_WINDOW: usize = 10;
const MAINTENANCE_INTERVAL_WINDOW: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    draw_calls: VecDeque<(usize, usize)>,
    render_durations: VecDeque<Duration>,
    redraw_intervals: VecDeque<Duration>,
    maintenance_intervals: VecDeque<Duration>,
}

impl Statistics {
    pub fn mean_world_draw_calls(&self) -> f32 {
        self.draw_calls.iter().map(|(wdc, _)| wdc).sum::<usize>() as f32 / DRAW_CALL_WINDOW as f32
    }

    pub fn mean_ui_draw_calls(&self) -> f32 {
        self.draw_calls.iter().map(|(_, udc)| udc).sum::<usize>() as f32 / DRAW_CALL_WINDOW as f32
    }

    pub fn mean_render_duration(&self) -> Duration {
        self.render_durations.iter().sum::<Duration>() / RENDER_DURATION_WINDOW as u32
    }

    pub fn mean_redraw_interval(&self) -> Duration {
        self.redraw_intervals.iter().sum::<Duration>() / REDRAW_INTERVAL_WINDOW as u32
    }

    pub fn mean_maintenance_interval(&self) -> Duration {
        self.maintenance_intervals.iter().sum::<Duration>() / MAINTENANCE_INTERVAL_WINDOW as u32
    }

    pub fn update_draw_calls(&mut self, world_draw_calls: usize, ui_draw_calls: usize) {
        self.draw_calls.push_front((world_draw_calls, ui_draw_calls));
        if self.draw_calls.len() > DRAW_CALL_WINDOW {
            self.draw_calls.truncate(DRAW_CALL_WINDOW);
        }
    }

    pub fn update_render_durations(&mut self, render_duration: Duration) {
        self.render_durations.push_front(render_duration);
        if self.render_durations.len() > RENDER_DURATION_WINDOW {
            self.render_durations.truncate(RENDER_DURATION_WINDOW);
        }
    }

    pub fn update_redraw_intervals(&mut self, redraw_interval: Duration) {
        self.redraw_intervals.push_front(redraw_interval);
        if self.redraw_intervals.len() > REDRAW_INTERVAL_WINDOW {
            self.redraw_intervals.truncate(REDRAW_INTERVAL_WINDOW);
        }
    }

    pub fn update_maintenance_intervals(&mut self, maintenance_interval: Duration) {
        self.maintenance_intervals.push_front(maintenance_interval);
        if self.maintenance_intervals.len() > MAINTENANCE_INTERVAL_WINDOW {
            self.maintenance_intervals.truncate(MAINTENANCE_INTERVAL_WINDOW);
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            draw_calls: VecDeque::with_capacity(DRAW_CALL_WINDOW),
            render_durations: VecDeque::with_capacity(RENDER_DURATION_WINDOW),
            redraw_intervals: VecDeque::with_capacity(REDRAW_INTERVAL_WINDOW),
            maintenance_intervals: VecDeque::with_capacity(MAINTENANCE_INTERVAL_WINDOW),
        }
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Loop and Render Stats:\nWorld draw calls (mean): {}\nRender duration (mean): {}\nRedraw interval (mean): {}\nMaintenance interval (mean): {}\n",
               self.mean_world_draw_calls(),
               humantime::format_duration(self.mean_render_duration()),
               humantime::format_duration(self.mean_redraw_interval()),
               humantime::format_duration(self.mean_maintenance_interval()),
        )
    }
}

impl Resource for Statistics {}

impl<D> WithDependencies<D> for Statistics {
    #[tracing::instrument]
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
