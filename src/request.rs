use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Request {
    Countdown { name: String, dur: u64, step: u64 },
    Stopwatch { name: String, step: u64 },
    Pause(String),
    Halt(String),
    Resume(String),
    Report,
    Quit,
}
