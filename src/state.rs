use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Deserialize, Serialize, Hash, Eq, PartialEq, Copy, Clone)]
pub enum State {
    Running,
    Paused,
    Halted,
}

impl State {
    pub fn is_paused(self) -> bool {
        if let State::Paused = self {
            true
        } else {
            false
        }
    }

    pub fn is_halted(self) -> bool {
        if let State::Halted = self {
            true
        } else {
            false
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Halted => write!(f, "halted"),
        }
    }
}
