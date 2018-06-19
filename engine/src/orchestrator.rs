use ecs::world::WorldTrait;
use failure::Error;
use file_manipulation::{FileError, VerifyPath};
use std::cmp;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Orchestrator<W>
where
    W: Default + WorldTrait,
{
    pub world: W,
    resource_path: PathBuf,
    delta_time: Duration,
    max_frame_time: Duration,
}

impl<W> Orchestrator<W>
where
    W: Default + WorldTrait,
{
    pub fn new(
        resource_path: &Path,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, FileError> {
        let rp = resource_path.to_path_buf().ensure_accessible_directory()?;

        Ok(Orchestrator {
            world: W::default(),
            resource_path: rp,
            delta_time: delta_time,
            max_frame_time: max_frame_time,
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
                self.world.fixed_update(&fixed_game_time, &self.delta_time)?;
                accumulator -= self.delta_time;
                fixed_game_time += self.delta_time;
            }

            self.world.update(&dynamic_game_time, &frame_time)?;
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
    use super::*;
    use ecs::mock::MockWorld;
    use std::env;
    use tempfile::NamedTempFile;

    /// Danger! This test works with thread::sleep() to test fixed loop timing. Note that the
    /// estimate of update calls is not always accurate, that's why this test is fuzzy by +/-1
    /// iteration. Because of this, the test will bust quickcheck's shrinking algorithm.
    fn check_fixed_update_calls(
        iterations: u32,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> bool {
        let base = env::temp_dir();
        let render_duration = Duration::from_millis(20);

        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.max_iterations = iterations as usize + 1;
        o.world.render_duration = Some(render_duration);

        let start_time = Instant::now();
        o.run(Some(iterations as usize)).unwrap();
        let total_frame_time = start_time.elapsed();
        let mut fixed_update_calls: u32 = 0;
        while delta_time * (fixed_update_calls + 1) <= total_frame_time {
            fixed_update_calls += 1;
        }

        let abs_error = (fixed_update_calls as f64 - o.world.fixed_update_calls as f64).abs();
        let rel_error = (fixed_update_calls as f64 - o.world.fixed_update_calls as f64)
            / fixed_update_calls as f64;
        abs_error <= 1.0 || rel_error <= 0.1
    }

    #[test]
    fn create_orchestrator() {
        let r = Orchestrator::<MockWorld>::new(
            &env::temp_dir(),
            Default::default(),
            Default::default(),
        );
        assert_ok!(r);

        let r = Orchestrator::<MockWorld>::new(
            &PathBuf::from("blablablabla"),
            Default::default(),
            Default::default(),
        );
        assert_err!(r);

        let tf = NamedTempFile::new().unwrap();
        let r = Orchestrator::<MockWorld>::new(tf.path(), Default::default(), Default::default());
        assert_err!(r);
    }
    #[test]
    fn get_resource_path() {
        let dir_name = ".";

        let base = env::temp_dir();
        let tf = NamedTempFile::new_in(&base).unwrap();

        let o =
            Orchestrator::<MockWorld>::new(&base, Default::default(), Default::default()).unwrap();

        let r = o.get_file(dir_name, &tf.path().file_name().unwrap().to_string_lossy());
        assert_ok!(r);

        let r = r.unwrap();
        assert_eq!(
            r,
            tf.path(),
            "Expected the path '{}', got '{}' instead",
            tf.path().display(),
            r.display()
        );

        let r = o.get_file("blabla", &tf.path().file_name().unwrap().to_string_lossy());
        assert_err!(r);

        let r = o.get_file(dir_name, "blabla.a");
        assert_err!(r);

        let r = o.get_file(dir_name, "..");
        assert_err!(r);
    }
    #[test]
    fn run_orchestrator_unrestrained() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None).unwrap();
        assert_eq!(
            o.world.handle_events_calls, o.world.max_iterations,
            "Expected {} iterations, got {} instead",
            o.world.max_iterations, o.world.handle_events_calls
        );
    }
    #[test]
    fn check_fixed_update_calls_a() {
        assert!(check_fixed_update_calls(
            10,
            Duration::from_millis(100),
            Duration::from_millis(250)
        ));
    }
    #[test]
    fn check_fixed_update_calls_b() {
        assert!(check_fixed_update_calls(
            10,
            Duration::from_millis(50),
            Duration::from_millis(250)
        ));
    }
    #[test]
    fn check_fixed_update_calls_d() {
        assert!(check_fixed_update_calls(
            50,
            Duration::from_millis(100),
            Duration::from_millis(250)
        ));
    }
    #[test]
    fn check_fixed_update_calls_e() {
        assert!(check_fixed_update_calls(
            50,
            Duration::from_millis(50),
            Duration::from_millis(250)
        ));
    }
    #[test]
    fn check_fixed_update_calls_f() {
        assert!(check_fixed_update_calls(
            50,
            Duration::from_millis(10),
            Duration::from_millis(250)
        ));
    }
    #[test]
    fn fixed_update_error() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.fixed_update_error_out = true;
        o.world.render_duration = Some(Duration::from_millis(20));

        let r = o.run(None);
        assert_err!(r);
    }
    quickcheck! {
        fn check_update_calls(iterations: usize) -> bool {
            let base = env::temp_dir();
            let delta_time = Duration::from_millis(50);
            let max_frame_time = Duration::from_millis(250);
            let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
            o.world.max_iterations = iterations + 1;

            o.run(Some(iterations)).unwrap();
            o.world.update_calls == iterations
        }
    }
    #[test]
    fn update_error() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.update_error_out = true;

        let r = o.run(None);
        assert_err!(r);
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
        assert_err!(r);
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
        assert_err!(r);
    }
    #[test]
    fn check_fixed_update_arguments() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<MockWorld>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None).unwrap();
        let mut last_time = Duration::default();
        assert!(o.world.fixed_update_arguments.iter().all(|&(t, dt)| {
            let temp = ((t - last_time) == delta_time) && (dt == delta_time);
            last_time = t;
            temp
        }));
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
