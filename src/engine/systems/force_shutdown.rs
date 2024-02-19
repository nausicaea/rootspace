#[cfg(not(test))]
use std::process;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use log::debug;
#[cfg(not(test))]
use log::error;
#[cfg(not(test))]
use log::info;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use crate::ecs::event_queue::EventQueue;
use crate::ecs::event_queue::receiver_id::ReceiverId;
use crate::ecs::resources::Resources;
use crate::ecs::system::System;
use crate::ecs::with_resources::WithResources;
use crate::engine::events::engine_event::EngineEvent;
use crate::engine::events::window_event::WindowEvent;

#[derive(Debug)]
pub struct ForceShutdown {
    ctrlc_triggered: Arc<AtomicUsize>,
    receiver: ReceiverId<WindowEvent>,
}

impl WithResources for ForceShutdown {
    fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let ctrlc_triggered = Arc::new(AtomicUsize::new(0));
        #[cfg(not(test))]
        {
            let trigger = ctrlc_triggered.clone();
            let result = ctrlc::set_handler(move || {
                let previous = trigger.fetch_add(1, Ordering::SeqCst);
                if previous > 0 {
                    info!("Force-quitting the application");
                    process::exit(1);
                }
            });
            if let Err(e) = result {
                error!("Unable to set a Ctrl-C handler: {}", e);
            }
        }

        let receiver = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();

        Ok(ForceShutdown {
            ctrlc_triggered,
            receiver,
        })
    }
}

impl System for ForceShutdown {
    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            debug!("Recently caught a termination signal");
            res.borrow_mut::<EventQueue<EngineEvent>>()
                .send(EngineEvent::AbortRequested);
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }

        let events = res.borrow_mut::<EventQueue<WindowEvent>>().receive(&self.receiver);
        for event in events {
            match event {
                WindowEvent::CloseRequested => {
                    debug!("User requested abort by closing the window");
                    res.borrow_mut::<EventQueue<EngineEvent>>()
                        .send(EngineEvent::AbortRequested);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            ..
                        },
                    ..
                } => {
                    debug!("User requested abort by pressing Q");
                    res.borrow_mut::<EventQueue<EngineEvent>>()
                        .send(EngineEvent::AbortRequested);
                }
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::registry::{End, SystemRegistry};
    use crate::ecs::world::World;
    use crate::Reg;
    use super::*;

    #[test]
    fn force_shutdown_reg_macro() {
        type _SR = Reg![ForceShutdown];
    }

    #[test]
    fn force_shutdown_system_registry() {
        let res = Resources::with_dependencies::<Reg![EventQueue<WindowEvent>], _>(&()).unwrap();
        let _rr = SystemRegistry::push(End, ForceShutdown::with_res(&res).unwrap());
    }

    #[test]
    fn force_shutdown_world() {
        let _w = World::with_dependencies::<Reg![EventQueue<WindowEvent>], Reg![], Reg![ForceShutdown], Reg![], _>(&())
            .unwrap();
    }
}
