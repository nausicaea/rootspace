#[cfg(not(test))]
use ctrlc;
use ecs::{System, Resources, EventManager};
use crate::event::EngineEventTrait;
#[cfg(not(test))]
use std::process;
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

pub struct ForceShutdown<Evt> {
    ctrlc_triggered: Arc<AtomicUsize>,
    _evt: PhantomData<Evt>,
}

impl<Evt> Default for ForceShutdown<Evt> {
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
            _evt: PhantomData::default(),
        }
    }
}

impl<Evt> System for ForceShutdown<Evt>
where
    Evt: EngineEventTrait,
{
    fn run(&mut self, res: &mut Resources, _: &Duration, _: &Duration) {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            trace!("Recently caught a termination signal");
            res.get_mut::<EventManager<Evt>>().dispatch_later(Evt::new_shutdown());
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }
    }
}
