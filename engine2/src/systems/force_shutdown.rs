#[cfg(not(test))]
use std::process;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use ecs::{EventQueue, Resources, System, WithResources};
use log::debug;
#[cfg(not(test))]
use log::error;
#[cfg(not(test))]
use log::info;
use serde::{Deserialize, Serialize};

use crate::events::engine_event::EngineEvent;

#[derive(Debug, Serialize, Deserialize)]
pub struct ForceShutdown {
    #[serde(skip)]
    ctrlc_triggered: Arc<AtomicUsize>,
}

impl WithResources for ForceShutdown {
    fn with_res(_res: &Resources) -> Result<Self, anyhow::Error> {
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

        Ok(ForceShutdown { ctrlc_triggered })
    }
}

impl System for ForceShutdown {
    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            debug!("Recently caught a termination signal");
            res.borrow_mut::<EventQueue<EngineEvent>>()
                .send(EngineEvent::AboutToAbort);
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use ecs::{SystemRegistry, Reg, End, World};

    use super::*;

    #[test]
    fn force_shutdown_reg_macro() {
        type _SR = Reg![ForceShutdown];
    }

    #[test]
    fn force_shutdown_system_registry() {
        let res = Resources::with_dependencies::<Reg![], _>(&()).unwrap();
        let _rr = SystemRegistry::push(End, ForceShutdown::with_res(&res).unwrap());
    }

    #[test]
    fn force_shutdown_world() {
        let _w = World::with_dependencies::<Reg![], Reg![], Reg![ForceShutdown], Reg![], _>(&()).unwrap();
    }
}
