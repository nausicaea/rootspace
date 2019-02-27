use ecs::{Resource, WorldTrait};
use std::time::Duration;

#[derive(Debug)]
pub struct MockWorld {
    pub max_iterations: usize,
    pub render_duration: Option<Duration>,
    pub fixed_update_calls: usize,
    pub update_calls: usize,
    pub render_calls: usize,
    pub handle_events_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_arguments: Vec<(Duration, Duration)>,
    iterations: usize,
}

impl WorldTrait for MockWorld {
    fn clear(&mut self) {
        self.fixed_update_calls = 0;
        self.update_calls = 0;
        self.render_calls = 0;
        self.handle_events_calls = 0;
        self.fixed_update_arguments.clear();
        self.update_arguments.clear();
        self.render_arguments.clear();
        self.iterations = 0;
    }

    fn add_resource<R>(&mut self, _res: R) -> Option<R>
    where
        R: Resource,
    {
        None
    }

    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration) {
        self.fixed_update_calls += 1;
        self.fixed_update_arguments.push((time.clone(), delta_time.clone()));
    }

    fn update(&mut self, time: &Duration, delta_time: &Duration) {
        self.update_calls += 1;
        self.update_arguments.push((time.clone(), delta_time.clone()));
    }

    fn render(&mut self, time: &Duration, delta_time: &Duration) {
        self.render_calls += 1;
        self.render_arguments.push((time.clone(), delta_time.clone()));
    }

    fn handle_events(&mut self) -> bool {
        self.handle_events_calls += 1;
        self.iterations += 1;
        if self.iterations < self.max_iterations {
            true
        } else {
            false
        }
    }
}

impl Default for MockWorld {
    fn default() -> Self {
        MockWorld {
            max_iterations: 1,
            render_duration: Some(Duration::from_millis(20)),
            fixed_update_calls: 0,
            update_calls: 0,
            render_calls: 0,
            handle_events_calls: 0,
            fixed_update_arguments: Vec::default(),
            update_arguments: Vec::default(),
            render_arguments: Vec::default(),
            iterations: 0,
        }
    }
}
