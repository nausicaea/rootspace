use crate::event::EngineEvent;
#[cfg(not(test))]
use ctrlc;
use ecs::{EventQueue, Resources, System};
#[cfg(any(test, debug_assertions))]
use log::debug;
#[cfg(not(test))]
use log::info;
#[cfg(not(test))]
use log::error;
#[cfg(not(test))]
use std::process;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

pub struct ForceShutdown {
    ctrlc_triggered: Arc<AtomicUsize>,
}

impl Default for ForceShutdown {
    fn default() -> Self {
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

        ForceShutdown { ctrlc_triggered }
    }
}

impl System for ForceShutdown {
    fn name(&self) -> &'static str {
        stringify!(ForceShutdown)
    }

    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            #[cfg(any(test, debug_assertions))]
            debug!("Recently caught a termination signal");
            res.borrow_mut::<EventQueue<EngineEvent>>().send(EngineEvent::Shutdown);
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }
    }
}
