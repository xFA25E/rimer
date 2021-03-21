use super::state::State;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
    time::Duration,
};

#[derive(Deserialize, Serialize, Eq)]
pub struct Snapshot {
    pub name: String,
    pub duration: Duration,
    pub elapsed: Duration,
    pub state: State,
    pub arg: String,
}

impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Snapshot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.name,
            self.elapsed.as_secs(),
            self.duration.as_secs(),
            self.state,
            self.arg
        )
    }
}
