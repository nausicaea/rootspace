use failure::Error;
use std::{thread, time::Duration};
use crate::world::WorldTrait;

#[derive(Debug)]
pub struct MockWorld {
    pub max_iterations: usize,
    pub render_duration: Option<Duration>,
    pub fixed_update_error_out: bool,
    pub fixed_update_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_error_out: bool,
    pub update_calls: usize,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_error_out: bool,
    pub render_calls: usize,
    pub render_arguments: Vec<(Duration, Duration)>,
    pub handle_events_error_out: bool,
    pub handle_events_calls: usize,
}

impl Default for MockWorld {
    fn default() -> Self {
        MockWorld {
            max_iterations: 10,
            render_duration: None,
            fixed_update_error_out: false,
            fixed_update_calls: 0,
            fixed_update_arguments: Vec::new(),
            update_error_out: false,
            update_calls: 0,
            update_arguments: Vec::new(),
            render_error_out: false,
            render_calls: 0,
            render_arguments: Vec::new(),
            handle_events_error_out: false,
            handle_events_calls: 0,
        }
    }
}

impl WorldTrait for MockWorld {
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.fixed_update_arguments.push((*time, *delta_time));
        self.fixed_update_calls += 1;
        if self.fixed_update_error_out {
            Err(format_err!("MockWorld.update() had an error."))
        } else {
            Ok(())
        }
    }
    fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.update_arguments.push((*time, *delta_time));
        self.update_calls += 1;
        if self.update_error_out {
            Err(format_err!("MockWorld.update() had an error."))
        } else {
            Ok(())
        }
    }
    fn render(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
        self.render_arguments.push((*time, *delta_time));
        self.render_calls += 1;

        if let Some(d) = self.render_duration {
            thread::sleep(d);
        }
        if self.render_error_out {
            Err(format_err!("MockWorld.render() had an error."))
        } else {
            Ok(())
        }
    }
    fn handle_events(&mut self) -> Result<bool, Error> {
        self.handle_events_calls += 1;
        if self.handle_events_error_out {
            Err(format_err!("MockWorld.handle_events() had an error."))
        } else {
            Ok(self.handle_events_calls < self.max_iterations)
        }
    }
}
