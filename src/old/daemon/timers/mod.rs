mod timer;

pub use timer::TimerUpdater;

use timer::Timer;

use std::{
    fmt::{self, Display},
    sync::{Arc, Mutex},
};

pub struct Timers {
    timers: Vec<Timer>,
    last_update: Arc<Mutex<u64>>,
    updater: Arc<TimerUpdater>,
}

macro_rules! timers_sender {
    ($func:ident) => {
        pub fn $func(&mut self, name: String) {
            if let Ok(inx) = self.search(&name) {
                self.timers[inx].$func()
            }
        }
    }
}

impl Timers {
    pub fn new(updater: TimerUpdater) -> Self {
        Self {
            timers: Vec::new(),
            last_update: Arc::new(Mutex::new(0)),
            updater: Arc::new(updater),
        }
    }

    fn remove_halted(&mut self) {
        self.timers.retain(|elm| !elm.is_halted())
    }

    pub fn add_countdown(&mut self, name: String, dur: u64, step: u64) {
        if let Err(inx) = self.search(&name) {
            self.timers.insert(
                inx,
                Timer::countdown(
                    name,
                    dur,
                    step,
                    Arc::clone(&self.updater),
                    Arc::clone(&self.last_update),
                ),
            );
        }
    }

    pub fn add_stopwatch(&mut self, name: String, step: u64) {
        if let Err(inx) = self.search(&name) {
            self.timers.insert(
                inx,
                Timer::stopwatch(
                    name,
                    step,
                    Arc::clone(&self.updater),
                    Arc::clone(&self.last_update),
                ),
            );
        }
    }

    fn search(&mut self, name: &str) -> Result<usize, usize> {
        self.remove_halted();
        self.timers.binary_search_by(|elm| elm.name().cmp(name))
    }

    timers_sender!(halt);
    timers_sender!(pause);
    timers_sender!(resume);
}

impl Display for Timers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for timer in self.timers.iter().filter(|e| !e.is_halted()) {
            writeln!(f, "{}", timer)?;
        }
        Ok(())
    }
}

impl Drop for Timers {
    fn drop(&mut self) {
        self.timers.clear();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
