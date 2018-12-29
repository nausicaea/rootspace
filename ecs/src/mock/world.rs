use std::{thread, time::Duration};
use crate::world::WorldTrait;

#[derive(Debug)]
pub struct MockWorld {
    pub max_iterations: usize,
    pub render_duration: Option<Duration>,
    pub fixed_update_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_calls: usize,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_calls: usize,
    pub render_arguments: Vec<(Duration, Duration)>,
    pub handle_events_calls: usize,
}

impl Default for MockWorld {
    fn default() -> Self {
        MockWorld {
            max_iterations: 10,
            render_duration: None,
            fixed_update_calls: 0,
            fixed_update_arguments: Vec::new(),
            update_calls: 0,
            update_arguments: Vec::new(),
            render_calls: 0,
            render_arguments: Vec::new(),
            handle_events_calls: 0,
        }
    }
}

impl WorldTrait for MockWorld {
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration) {
        self.fixed_update_arguments.push((*time, *delta_time));
        self.fixed_update_calls += 1;
    }

    fn update(&mut self, time: &Duration, delta_time: &Duration) {
        self.update_arguments.push((*time, *delta_time));
        self.update_calls += 1;
    }

    fn render(&mut self, time: &Duration, delta_time: &Duration) {
        self.render_arguments.push((*time, *delta_time));
        self.render_calls += 1;

        if let Some(d) = self.render_duration {
            thread::sleep(d);
        }
    }

    fn handle_events(&mut self) -> bool {
        self.handle_events_calls += 1;
        self.handle_events_calls < self.max_iterations
    }
}
