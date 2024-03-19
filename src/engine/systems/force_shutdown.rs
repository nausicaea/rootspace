#[cfg(not(test))]
use std::process;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::NamedKey,
};

use crate::{
    ecs::{
        event_queue::{receiver_id::ReceiverId, EventQueue},
        resources::Resources,
        system::System,
        with_resources::WithResources,
    },
    engine::events::engine_event::EngineEvent,
};

#[derive(Debug)]
pub struct ForceShutdown {
    ctrlc_triggered: Arc<AtomicUsize>,
    receiver: ReceiverId<WindowEvent>,
}

impl WithResources for ForceShutdown {
    async fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let ctrlc_triggered = Arc::new(AtomicUsize::new(0));
        #[cfg(not(test))]
        {
            let trigger = ctrlc_triggered.clone();
            let result = ctrlc::set_handler(move || {
                let previous = trigger.fetch_add(1, Ordering::SeqCst);
                if previous > 0 {
                    tracing::info!("Force-quitting the application");
                    process::exit(1);
                }
            });
            if let Err(e) = result {
                tracing::error!("Unable to set a Ctrl-C handler: {}", e);
            }
        }

        let receiver = res.write::<EventQueue<WindowEvent>>().subscribe::<Self>();

        Ok(ForceShutdown {
            ctrlc_triggered,
            receiver,
        })
    }
}

#[async_trait]
impl System for ForceShutdown {
    async fn run(&mut self, res: &Resources, _: Duration, _: Duration) {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            tracing::debug!("User requested to exit by SIGINT");
            res.write::<EventQueue<EngineEvent>>().send(EngineEvent::Exit);
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }

        let events = res.write::<EventQueue<WindowEvent>>().receive(&self.receiver);
        for event in events {
            match event {
                WindowEvent::CloseRequested => {
                    tracing::debug!("User requested to exit by closing the window");
                    res.write::<EventQueue<EngineEvent>>().send(EngineEvent::Exit);
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Released,
                            logical_key: winit::keyboard::Key::Named(NamedKey::Exit),
                            ..
                        },
                    ..
                } => {
                    tracing::debug!("User requested to exit by pressing Cmd-Q");
                    res.write::<EventQueue<EngineEvent>>().send(EngineEvent::Exit);
                }
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ecs::{
            registry::{End, SystemRegistry},
            world::World,
        },
        Reg,
    };

    #[test]
    fn force_shutdown_reg_macro() {
        type _SR = Reg![ForceShutdown];
    }

    #[tokio::test]
    async fn force_shutdown_system_registry() {
        let res = Resources::with_dependencies::<Reg![EventQueue<WindowEvent>], _>(&())
            .await
            .unwrap();
        let _rr = SystemRegistry::push(End, ForceShutdown::with_res(&res).await.unwrap());
    }

    #[tokio::test]
    async fn force_shutdown_world() {
        let _w =
            World::with_dependencies::<Reg![EventQueue<WindowEvent>], Reg![], Reg![ForceShutdown], (), Reg![], _>(&())
                .await
                .unwrap();
    }
}
