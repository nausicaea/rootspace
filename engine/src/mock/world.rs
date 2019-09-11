use ecs::{Entity, LoopStage, Persistence, RegAdd, Registry, ResourcesTrait, WorldTrait};
use serde::{de::Deserializer, ser::Serializer};
use std::{marker::PhantomData, time::Duration};

#[derive(Debug)]
pub struct MockWorld<RR> {
    pub max_iterations: usize,
    pub render_duration: Option<Duration>,
    pub fixed_update_calls: usize,
    pub update_calls: usize,
    pub render_calls: usize,
    pub maintain_calls: usize,
    pub fixed_update_arguments: Vec<(Duration, Duration)>,
    pub update_arguments: Vec<(Duration, Duration)>,
    pub render_arguments: Vec<(Duration, Duration)>,
    iterations: usize,
    _rr: PhantomData<RR>,
}

impl<RR> ResourcesTrait<RR> for MockWorld<RR>
where
    RR: Registry,
{
    type ResourceRegistry = RegAdd![RR];

    fn serialize<S>(&self, _serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }

    fn deserialize<'de, D>(&mut self, _deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }

    fn clear(&mut self, _persistence: Persistence) {
        self.fixed_update_calls = 0;
        self.update_calls = 0;
        self.render_calls = 0;
        self.maintain_calls = 0;
        self.fixed_update_arguments.clear();
        self.update_arguments.clear();
        self.render_arguments.clear();
        self.iterations = 0;
    }

    fn add_resource<R, S>(&mut self, _res: R, _settings: S) {}

    fn get_resource_mut<R>(&mut self) -> &mut R {
        unimplemented!()
    }

    fn create_entity(&mut self) -> Entity {
        unimplemented!()
    }

    fn add_component<C>(&mut self, _entity: Entity, _component: C) {
        unimplemented!()
    }
}

impl<RR> WorldTrait for MockWorld<RR> {
    fn add_system<S>(&mut self, _stage: LoopStage, _system: S) {
        unimplemented!()
    }

    fn get_system<S>(&self, _stage: LoopStage) -> Option<&S> {
        unimplemented!()
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

    fn maintain(&mut self) -> bool {
        self.maintain_calls += 1;
        self.iterations += 1;
        if self.iterations < self.max_iterations {
            true
        } else {
            false
        }
    }
}

impl<RR> Default for MockWorld<RR> {
    fn default() -> Self {
        MockWorld {
            max_iterations: 1,
            render_duration: Some(Duration::from_millis(20)),
            fixed_update_calls: 0,
            update_calls: 0,
            render_calls: 0,
            maintain_calls: 0,
            fixed_update_arguments: Vec::default(),
            update_arguments: Vec::default(),
            render_arguments: Vec::default(),
            iterations: 0,
            _rr: PhantomData::default(),
        }
    }
}
