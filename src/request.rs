use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize, Serialize)]
pub enum Request {
    Add {
        name: String,
        duration: Duration,
        step: Duration,
        arg: String,
    },
    Pause {
        name: String,
    },
    Halt {
        name: String,
    },
    Resume {
        name: String,
    },
    Report {
        json: bool,
    },
    Quit,
}
