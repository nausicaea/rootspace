use crate::event::{EventManagerTrait, EventTrait};
use std::{marker::PhantomData, time::Duration};
use crate::system::{System, EventHandlerSystem};
use crate::loop_stage::LoopStage;

/// A World must perform actions for four types of calls.
pub trait WorldTrait {
    /// The fixed update method is supposed to be called from the main loop at fixed time
    /// intervals.
    ///
    /// # Aruments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `fixed_update`.
    fn fixed_update(&mut self, time: &Duration, delta_time: &Duration);
    /// The dynamic update method is supposed to be called from the main loop just before the
    /// render call.
    ///
    /// # Aruments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `update`.
    fn update(&mut self, time: &Duration, delta_time: &Duration);
    /// The render method is supposed to be called when a re-draw of the graphical representation
    /// is desired.
    ///
    /// # Aruments
    ///
    /// * `time` - Interpreted as the current game time.
    /// * `delta_time` - Interpreted as the time interval between calls to `render`.
    fn render(&mut self, time: &Duration, delta_time: &Duration);
    /// The handle events method is supposed to be called when pending events or messages should be
    /// handled by the connected systems. If this method returns `Ok(true)`, the execution of the
    /// main loop shall continue, otherwise it shall abort.
    fn handle_events(&mut self) -> bool;
}

/// This is the default implementation of the `WorldTrait` provided by this library.
pub struct World<E, C> {
    /// The context must be capable of managing events and messages. Additionally, any behaviour
    /// may be added.
    pub context: C,
    fixed_update_systems: Vec<Box<System<C>>>,
    update_systems: Vec<Box<System<C>>>,
    render_systems: Vec<Box<System<C>>>,
    event_handler_systems: Vec<Box<EventHandlerSystem<C, E>>>,
    _e: PhantomData<E>,
}

impl<E, C> World<E, C>
where
    E: EventTrait,
    C: Default + EventManagerTrait<E>,
{
    pub fn add_system<S>(&mut self, stage: LoopStage, system: S) where S: System<C> + 'static {
        let sys = Box::new(system);
        match stage {
            LoopStage::FixedUpdate => self.fixed_update_systems.push(sys),
            LoopStage::Update => self.update_systems.push(sys),
            LoopStage::Render => self.render_systems.push(sys),
        }
    }

    pub fn add_event_handler_system<H>(&mut self, system: H) where H: EventHandlerSystem<C, E> + 'static {
        self.event_handler_systems.push(Box::new(system))
    }
}

impl<E, C> Default for World<E, C>
where
    E: EventTrait,
    C: Default + EventManagerTrait<E>,
{
    fn default() -> Self {
        World {
            context: Default::default(),
            fixed_update_systems: Vec::default(),
            update_systems: Vec::default(),
            render_systems: Vec::default(),
            event_handler_systems: Vec::default(),
            _e: PhantomData::default(),
        }
    }
}

impl<E, C> WorldTrait for World<E, C>
where
    E: EventTrait,
    C: Default + EventManagerTrait<E>,
{
    fn fixed_update(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.fixed_update_systems {
            system.run(&mut self.context, t, dt);
        }
    }

    fn update(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.update_systems {
            system.run(&mut self.context, t, dt);
        }
    }

    fn render(&mut self, t: &Duration, dt: &Duration) {
        for system in &mut self.render_systems {
            system.run(&mut self.context, t, dt);
        }
    }

    fn handle_events(&mut self) -> bool {
        let systems = &mut self.event_handler_systems;

        self.context.handle_events(|ctx, event| {
            let mut statuses: Vec<bool> = Vec::with_capacity(systems.len());

            for system in systems.iter_mut() {
                if event.matches_filter(system.get_event_filter()) {
                    statuses.push(system.run(ctx, event));
                }
            }

            Ok(statuses.iter().all(|s| *s))
        }).unwrap()
    }
}
