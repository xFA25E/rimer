use std::{
    fmt::{self, Display},
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

pub enum TimerState {
    Running = 0,
    Paused = 1,
    Halted = 2,
}

impl TimerState {
    pub fn is_halted(&self) -> bool {
        if let Self::Halted = self {
            true
        } else {
            false
        }
    }
}

impl Display for TimerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Halted => write!(f, "halted"),
        }
    }
}

pub struct TimerStateFlag(AtomicUsize);

impl TimerStateFlag {
    pub fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn resume(&self) {
        self.0.store(TimerState::Running as usize, Relaxed)
    }

    pub fn pause(&self) {
        self.0.store(TimerState::Paused as usize, Relaxed)
    }

    pub fn halt(&self) {
        self.0.store(TimerState::Halted as usize, Relaxed)
    }

    pub fn get(&self) -> TimerState {
        match self.0.load(Relaxed) {
            0 => TimerState::Running,
            1 => TimerState::Paused,
            2 => TimerState::Halted,
            _ => unreachable!(),
        }
    }

    pub fn is_halted(&self) -> bool {
        self.get().is_halted()
    }
}
