use crate::{
    components::{model::Model, ui_model::UiModel},
    event::EngineEvent,
    file_manipulation::{FileError, VerifyPath},
    graphics::BackendTrait,
    resources::{Backend, SceneGraph},
};
use ecs::{RegAdd, Registry, LoopStage, System, Component, Entity, EventQueue, Persistence, Resource, ReceiverId, ResourcesTrait, WorldTrait};
use std::{
    cmp,
    marker::PhantomData,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use typename::TypeName;
use serde::{de::Deserializer, ser::Serializer};

pub type JoinedRegistry<RR> = RegAdd![
    SceneGraph<UiModel>,
    SceneGraph<Model>,
    EventQueue<EngineEvent>,
    RR
];

#[derive(Debug)]
pub struct Orchestrator<B, RR, W> {
    pub world: W,
    resource_path: PathBuf,
    delta_time: Duration,
    max_frame_time: Duration,
    receiver: ReceiverId<EngineEvent>,
    _b: PhantomData<B>,
    _rr: PhantomData<RR>,
}

impl<B, RR, W> Orchestrator<B, RR, W>
where
    B: BackendTrait,
    RR: Registry + Default,
    W: Default + WorldTrait + ResourcesTrait<JoinedRegistry<RR>>,
{
    pub fn new<P: AsRef<Path>>(
        resource_path: P,
        delta_time: Duration,
        max_frame_time: Duration,
    ) -> Result<Self, FileError> {
        resource_path.ensure_extant_directory()?;

        let mut events: EventQueue<EngineEvent> = EventQueue::default();
        let receiver = events.subscribe();

        let mut world = W::default();
        world.add_resource(events, Persistence::Runtime);
        world.add_resource(Backend::<B>::default(), Persistence::Runtime);
        world.add_resource(SceneGraph::<Model>::default(), Persistence::Runtime);
        world.add_resource(SceneGraph::<UiModel>::default(), Persistence::Runtime);

        Ok(Orchestrator {
            world,
            resource_path: resource_path.as_ref().into(),
            delta_time,
            max_frame_time,
            receiver,
            _b: PhantomData::default(),
            _rr: PhantomData::default(),
        })
    }

    pub fn run(&mut self, iterations: Option<usize>) {
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
                self.world.fixed_update(&fixed_game_time, &self.delta_time);
                accumulator -= self.delta_time;
                fixed_game_time += self.delta_time;
            }

            self.world.update(&dynamic_game_time, &frame_time);
            self.world.render(&dynamic_game_time, &frame_time);
            running = self.world.maintain();
            i += 1;
        }
    }

    pub fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        self.world.serialize::<S>(serializer)
    }

    pub fn deserialize<'de, D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        self.world.deserialize::<D>(deserializer)
    }

    pub fn add_resource<R: Resource + TypeName>(&mut self, res: R, persistence: Persistence) {
        self.world.add_resource::<R>(res, persistence)
    }

    pub fn get_resource_mut<R: Resource + TypeName>(&mut self) -> &mut R {
        self.world.get_resource_mut::<R>()
    }

    pub fn create_entity(&mut self) -> Entity {
        self.world.create_entity()
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component + TypeName,
        C::Storage: TypeName,
    {
        self.world.add_component::<C>(entity, component)
    }

    pub fn reset(&mut self) {
        self.world.clear(Persistence::None);
    }

    pub fn add_system<S>(&mut self, stage: LoopStage, system: S)
    where
        S: System,
    {
        self.world.add_system::<S>(stage, system)
    }

    pub fn file(&self, folder: &str, file: &str) -> Result<PathBuf, FileError> {
        let path = self.resource_path.join(folder).join(file);
        path.ensure_extant_file()?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::Reg;
    use crate::{graphics::headless::HeadlessBackend, mock::MockWorld};
    use proptest::prelude::*;
    use std::env;
    use tempfile::NamedTempFile;

    type TestRegistry = Reg![];

    /// Danger! This test works with thread::sleep() to test fixed loop timing. Note that the
    /// estimate of update calls is not always accurate, that's why this test is fuzzy by +/-1
    /// iteration. Because of this, the test will bust quickcheck's shrinking algorithm.
    fn check_fixed_update_calls(iterations: u32, delta_time: Duration, max_frame_time: Duration) -> bool {
        let base = env::temp_dir();
        let render_duration = Duration::from_millis(20);

        let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();
        o.world.max_iterations = iterations as usize + 1;
        o.world.render_duration = Some(render_duration);

        let start_time = Instant::now();
        o.run(Some(iterations as usize));
        let total_frame_time = start_time.elapsed();
        let mut fixed_update_calls: u32 = 0;
        while delta_time * (fixed_update_calls + 1) <= total_frame_time {
            fixed_update_calls += 1;
        }

        let abs_error = (fixed_update_calls as f64 - o.world.fixed_update_calls as f64).abs();
        let rel_error = (fixed_update_calls as f64 - o.world.fixed_update_calls as f64) / fixed_update_calls as f64;
        abs_error <= 1.0 || rel_error <= 0.1
    }

    #[test]
    fn create_orchestrator() {
        let r =
            Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&env::temp_dir(), Default::default(), Default::default());
        assert!(r.is_ok());

        let r = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new("blablablabla", Default::default(), Default::default());
        assert!(r.is_err());

        let tf = NamedTempFile::new().unwrap();
        let r = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(tf.path(), Default::default(), Default::default());
        assert!(r.is_err());
    }

    #[test]
    fn get_resource_path() {
        let dir_name = ".";

        let base = env::temp_dir();
        let tf = NamedTempFile::new_in(&base).unwrap();

        let o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, Default::default(), Default::default()).unwrap();

        let r = o.file(dir_name, &tf.path().file_name().unwrap().to_string_lossy());
        assert!(r.is_ok());

        let r = r.unwrap();
        assert_eq!(
            r,
            tf.path(),
            "Expected the path '{}', got '{}' instead",
            tf.path().display(),
            r.display()
        );

        let r = o.file("blabla", &tf.path().file_name().unwrap().to_string_lossy());
        assert!(r.is_err());

        let r = o.file(dir_name, "blabla.a");
        assert!(r.is_err());

        let r = o.file(dir_name, "..");
        assert!(r.is_err());
    }

    #[test]
    fn run_orchestrator_unrestrained() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None);
        assert_eq!(
            o.world.max_iterations, o.world.maintain_calls,
            "Expected {} iterations, got {} instead",
            o.world.max_iterations, o.world.maintain_calls
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

    proptest! {
        #[test]
        fn check_update_calls(iterations in 0usize..1000) {
            let base = env::temp_dir();
            let delta_time = Duration::from_millis(50);
            let max_frame_time = Duration::from_millis(250);
            let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();
            o.world.max_iterations = iterations + 1;

            o.run(Some(iterations));
            prop_assert_eq!(o.world.update_calls, iterations)
        }

        #[test]
        fn check_render_calls(iterations in 0usize..1000) {
            let base = env::temp_dir();
            let delta_time = Duration::from_millis(50);
            let max_frame_time = Duration::from_millis(250);
            let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();
            o.world.max_iterations = iterations + 1;

            o.run(Some(iterations));
            prop_assert_eq!(o.world.render_calls, iterations)
        }

        #[test]
        fn check_maintain_calls(iterations in 0usize..1000) {
            let base = env::temp_dir();
            let delta_time = Duration::from_millis(50);
            let max_frame_time = Duration::from_millis(250);
            let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();
            o.world.max_iterations = iterations + 1;

            o.run(Some(iterations));
            prop_assert_eq!(o.world.maintain_calls, iterations)
        }
    }

    #[test]
    fn check_fixed_update_arguments() {
        let base = env::temp_dir();
        let delta_time = Duration::from_millis(50);
        let max_frame_time = Duration::from_millis(250);
        let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None);
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
        let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None);
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
        let mut o = Orchestrator::<HeadlessBackend, TestRegistry, MockWorld<JoinedRegistry<TestRegistry>>>::new(&base, delta_time, max_frame_time).unwrap();

        o.run(None);
        let mut last_time = Duration::default();
        assert!(o.world.render_arguments.iter().all(|&(t, dt)| {
            let temp = (t - last_time) == dt;
            last_time = t;
            temp
        }));
    }
}
