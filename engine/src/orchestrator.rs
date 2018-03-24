use std::cmp;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use failure::Error;
use ecs::world::WorldTrait;
use file_manipulation::{VerifyPath, FileError};

#[derive(Debug)]
pub struct Orchestrator<W> where W: WorldTrait + Default + Debug {
    resource_path: PathBuf,
    delta_time: Duration,
    max_frame_time: Duration,
    world: W,
}

impl<W> Orchestrator<W> where W: WorldTrait + Default + Debug {
    pub fn new(resource_path: &Path, delta_time: Duration, max_frame_time: Duration) -> Result<Self, FileError> {
        let rp = resource_path.to_path_buf()
            .ensure_accessible_directory()?;

        Ok(Orchestrator {
            resource_path: rp,
            delta_time: delta_time,
            max_frame_time: max_frame_time,
            world: W::default(),
        })
    }
    pub fn run(&mut self, iterations: Option<usize>) -> Result<(), Error> {
        let mut loop_time = Instant::now();
        let mut accumulator = Duration::default();
        let mut dynamic_game_time = Duration::default();
        let mut fixed_game_time = Duration::default();

        let mut i = 0;
        let mut running = true;
        while running && iterations.map(|max_iter| i < max_iter).unwrap_or(true) {
            let frame_time = cmp::min(loop_time.elapsed(), self.max_frame_time);
            loop_time = Instant::now();
            accumulator += frame_time;
            dynamic_game_time += frame_time;

            while accumulator >= self.delta_time {
                self.world.update(&fixed_game_time, &self.delta_time)?;
                accumulator -= self.delta_time;
                fixed_game_time += self.delta_time;
            }

            self.world.dynamic_update(&dynamic_game_time, &frame_time)?;
            self.world.render(&dynamic_game_time, &frame_time)?;
            running = self.world.handle_events()?;
            i += 1;
        }
        Ok(())
    }
    pub fn get_file(&self, folder: &str, file: &str) -> Result<PathBuf, FileError> {
        self.resource_path
            .join(folder)
            .join(file)
            .ensure_accessible_file()
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::thread;
    use tempfile::NamedTempFileOptions;
    use super::*;

    #[derive(Debug)]
    pub struct MockWorld {
        max_iterations: usize,
        render_duration: Option<Duration>,
        update_error_out: bool,
        update_calls: usize,
        update_arguments: Vec<(Duration, Duration)>,
        dynamic_update_error_out: bool,
        dynamic_update_calls: usize,
        dynamic_update_arguments: Vec<(Duration, Duration)>,
        render_error_out: bool,
        render_calls: usize,
        render_arguments: Vec<(Duration, Duration)>,
        handle_events_error_out: bool,
        handle_events_calls: usize,
    }

    impl Default for MockWorld {
        fn default() -> Self {
            MockWorld {
                max_iterations: 10,
                render_duration: None,
                update_error_out: false,
                update_calls: 0,
                update_arguments: Vec::new(),
                dynamic_update_error_out: false,
                dynamic_update_calls: 0,
                dynamic_update_arguments: Vec::new(),
                render_error_out: false,
                render_calls: 0,
                render_arguments: Vec::new(),
                handle_events_error_out: false,
                handle_events_calls: 0,
            }
        }
    }

    impl WorldTrait for MockWorld {
        fn update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.update_arguments.push((*time, *delta_time));
            self.update_calls += 1;
            if self.update_error_out {
                Err(format_err!("MockWorld.update() had an error."))
            } else {
                Ok(())
            }
        }
        fn dynamic_update(&mut self, time: &Duration, delta_time: &Duration) -> Result<(), Error> {
            self.dynamic_update_arguments.push((*time, *delta_time));
            self.dynamic_update_calls += 1;
            if self.dynamic_update_error_out {
                Err(format_err!("MockWorld.dynamic_update() had an error."))
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

    /// Danger! This test works with thread::sleep() to test fixed loop timing. Note that the
    /// estimate of update calls is not always accurate, that's why this test is fuzzy by +/-1
    /// iteration. Because of this, the test will bust quickcheck's shrinking algorithm.
    fn check_update_calls(iterations: u32, delta_time: Duration, max_frame_time: Duration) -> bool {
        let base = env::temp_dir();
        let render_duration = Duration::from_millis(20);

        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.max_iterations = iterations as usize + 1;
        o.world.render_duration = Some(render_duration);

        let start_time = Instant::now();
        o.run(Some(iterations as usize)).unwrap();
        let total_frame_time = start_time.elapsed();
        let mut update_calls: u32 = 0;
        while delta_time * (update_calls + 1) <= total_frame_time {
            update_calls += 1;
        }

        let abs_error = (update_calls as f64 - o.world.update_calls as f64).abs();
        let rel_error = (update_calls as f64 - o.world.update_calls as f64) / update_calls as f64;
        abs_error <= 1.0 || rel_error <= 0.1
    }

    #[test]
    fn create_orchestrator() {
        let r = Orchestrator::<MockWorld>::new(&env::temp_dir(), Default::default(), Default::default());
        assert!(r.is_ok(), "Expected an orchestrator instance, got the error '{}' instead", r.unwrap_err());

        let r = Orchestrator::<MockWorld>::new(&PathBuf::from("blablablabla"), Default::default(), Default::default());
        assert!(r.is_err(), "Expected an error, got an orchestrator instance instead");

        let tf = NamedTempFileOptions::new()
            .create()
            .unwrap();
        let r = Orchestrator::<MockWorld>::new(tf.path(), Default::default(), Default::default());
        assert!(r.is_err(), "Expected an error, got an orchestrator instance instead");
    }
    #[test]
    fn get_resource_path() {
        let dir_name = ".";

        let base = env::temp_dir();
        let tf = NamedTempFileOptions::new()
            .create_in(&base)
            .unwrap();

        let o = Orchestrator::<MockWorld>::new(&base, Default::default(), Default::default()).unwrap();

        let r = o.get_file(dir_name, &tf.path().file_name().unwrap().to_string_lossy());
        assert!(r.is_ok(), "Expected a path, got the error '{}' instead", r.unwrap_err());

        let r = r.unwrap();
        assert_eq!(r, tf.path(), "Expected the path '{}', got '{}' instead", tf.path().display(), r.display());

        let r = o.get_file("blabla", &tf.path().file_name().unwrap().to_string_lossy());
        assert!(r.is_err(), "Expected an error, got the path '{}' instead", r.unwrap().display());

        let r = o.get_file(dir_name, "blabla.a");
        assert!(r.is_err(), "Expected an error, got the path '{}' instead", r.unwrap().display());

        let r = o.get_file(dir_name, "..");
        assert!(r.is_err(), "Expected an error, got the path '{}' instead", r.unwrap().display());
    }
    #[test]
    fn run_orchestrator_unrestrained() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None).unwrap();
        assert_eq!(o.world.handle_events_calls, o.world.max_iterations, "Expected {} iterations, got {} instead", o.world.max_iterations, o.world.handle_events_calls);
    }
    #[test]
    fn check_update_calls_a() {
        assert!(check_update_calls(10, Duration::from_millis(100), Duration::from_millis(250)));
    }
    #[test]
    fn check_update_calls_b() {
        assert!(check_update_calls(10, Duration::from_millis(50), Duration::from_millis(250)));
    }
    #[test]
    fn check_update_calls_d() {
        assert!(check_update_calls(50, Duration::from_millis(100), Duration::from_millis(250)));
    }
    #[test]
    fn check_update_calls_e() {
        assert!(check_update_calls(50, Duration::from_millis(50), Duration::from_millis(250)));
    }
    #[test]
    fn check_update_calls_f() {
        assert!(check_update_calls(50, Duration::from_millis(10), Duration::from_millis(250)));
    }
    #[test]
    fn update_error() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.update_error_out = true;
        o.world.render_duration = Some(Duration::from_millis(20));

        let r = o.run(None);
        assert!(r.is_err());
    }
    quickcheck! {
        fn check_dynamic_update_calls(iterations: usize) -> bool {
            let base = env::temp_dir();
            let delta_time = Duration::from_millis(50);
            let max_frame_time = Duration::from_millis(250);
            let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
            o.world.max_iterations = iterations + 1;

            o.run(Some(iterations)).unwrap();
            o.world.dynamic_update_calls == iterations
        }
    }
    #[test]
    fn dynamic_update_error() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.dynamic_update_error_out = true;

        let r = o.run(None);
        assert!(r.is_err());
    }
    quickcheck! {
        fn check_render_calls(iterations: usize) -> bool {
            let base = env::temp_dir();
            let delta_time = Duration::from_millis(50);
            let max_frame_time = Duration::from_millis(250);
            let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
            o.world.max_iterations = iterations + 1;

            o.run(Some(iterations)).unwrap();
            o.world.render_calls == iterations
        }
    }
    #[test]
    fn render_error() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.render_error_out = true;

        let r = o.run(None);
        assert!(r.is_err());
    }
    quickcheck! {
        fn check_handle_events_calls(iterations: usize) -> bool {
            let base = env::temp_dir();
            let delta_time = Duration::from_millis(50);
            let max_frame_time = Duration::from_millis(250);
            let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
            o.world.max_iterations = iterations + 1;

            o.run(Some(iterations)).unwrap();
            o.world.handle_events_calls == iterations
        }
    }
    #[test]
    fn handle_events_error() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.handle_events_error_out = true;

        let r = o.run(None);
        assert!(r.is_err());
    }
    #[test]
    fn check_update_arguments() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None).unwrap();
        let mut last_time = Duration::default();
        assert!(o.world.update_arguments.iter().all(|&(t, dt)| {
            let temp = ((t - last_time) == delta_time) && (dt == delta_time);
            last_time = t;
            temp
        }));
    }
    #[test]
    fn check_dynamic_update_arguments() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None).unwrap();
        let mut last_time = Duration::default();
        assert!(o.world.dynamic_update_arguments.iter().all(|&(t, dt)| {
            let temp = (t - last_time) == dt;
            last_time = t;
            temp
        }));
    }
    #[test]
    fn check_render_arguments() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None).unwrap();
        let mut last_time = Duration::default();
        assert!(o.world.render_arguments.iter().all(|&(t, dt)| {
            let temp = (t - last_time) == dt;
            last_time = t;
            temp
        }));
    }
}
