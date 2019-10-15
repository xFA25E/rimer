use super::{state::TimerState, TimerKind};

use std::{process::Command, sync::Arc, thread};

pub struct TimerUpdater(Arc<String>);

impl TimerUpdater {
    pub fn new(s: String) -> Self {
        Self(Arc::new(s))
    }

    pub fn exec(&self, s: TimerSnapshot) {
        thread::spawn({
            let p = Arc::clone(&self.0);
            move || {
                Command::new(&*p)
                    .arg(&*s.name)
                    .arg(s.kind.to_string())
                    .arg(s.elapsed.to_string())
                    .arg(s.dur.to_string())
                    .arg(s.state.to_string())
                    .status()
            }
        });
    }
}

pub struct TimerSnapshot {
    pub name: Arc<String>,
    pub kind: TimerKind,
    pub elapsed: u64,
    pub dur: u64,
    pub state: TimerState,
}
