mod timers;

pub use timers::TimerUpdater;

use timers::Timers;

use crate::{request::Request, socket::listener};

use daemonize::Daemonize;

use std::{
    fs::{DirBuilder, File},
    io::Write,
};

macro_rules! econtinue {
    ( $x:expr ) => {
        match $x {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        }
    };
}

pub fn run(updater: TimerUpdater) -> Result<(), Box<dyn std::error::Error>> {
    start_daemon()?;
    let mut timers = Timers::new(updater);
    loop {
        let listener = listener()?;

        for mut stream in listener.incoming().filter_map(Result::ok) {
            let request = econtinue!(serde_json::from_reader(&stream));

            match request {
                Request::Countdown { name, dur, step } => {
                    if step != 0 {
                        timers.add_countdown(name, dur, step)
                    }
                }
                Request::Stopwatch { name, step } => {
                    if step != 0 {
                        timers.add_stopwatch(name, step)
                    }
                }
                Request::Pause(name) => timers.pause(name),
                Request::Halt(name) => timers.halt(name),
                Request::Resume(name) => timers.resume(name),
                Request::Report => econtinue!(write!(stream, "{}", timers)),
                Request::Quit => return Ok(()),
            }
        }
    }
}

fn start_daemon() -> Result<(), Box<dyn std::error::Error>> {
    Daemonize::new()
        .stderr({
            let mut p = dirs::cache_dir().unwrap();
            p.push("rimer");
            DirBuilder::new().recursive(true).create(&p)?;
            p.push("daemon.err");
            File::create(p)?
        })
        .start()?;
    Ok(())
}
