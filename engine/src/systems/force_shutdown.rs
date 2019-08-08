use crate::event::EngineEvent;
#[cfg(not(test))]
use ctrlc;
use ecs::{EventQueue, Resources, System};
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
            let r = ctrlc_triggered.clone();
            ctrlc::set_handler(move || {
                let previous = r.fetch_add(1, Ordering::SeqCst);
                if previous > 0 {
                    error!("Force-quitting the application");
                    process::exit(1);
                }
            })
            .expect("Unable to set a termination handler");
        }

        ForceShutdown {
            ctrlc_triggered,
        }
    }
}

impl System for ForceShutdown {
    fn name(&self) -> &'static str {
        "ForceShutdown"
    }

    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            trace!("Recently caught a termination signal");
            res.borrow_mut::<EventQueue<EngineEvent>>().send(EngineEvent::Shutdown);
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }
    }
}
