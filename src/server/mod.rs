mod timer;
mod updater;

use crate::{
    request::Request, response::Error, response::Response, snapshot as s,
    socket::listener,
};
use daemonize::Daemonize;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{DirBuilder, File},
    io::{Read, Write},
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc,
    },
};
use timer::Timer;
use updater::{self as u, Updater};

type Timers = HashMap<Arc<String>, Timer>;

struct Inner {
    updater: Updater,
    timers: Timers,
    halt_queue: Sender<Arc<String>>,
    halt_recv: Receiver<Arc<String>>,
    report_queue: Sender<u::Snapshot>,
    report_recv: Receiver<u::Snapshot>,
}

pub fn run(command: String) -> Result<(), Box<dyn std::error::Error>> {
    daemonize()?;

    let (halt_queue, halt_recv) = channel();
    let (report_queue, report_recv) = channel();
    let mut inner = Inner {
        updater: Updater::spawn(command),
        timers: HashMap::new(),
        halt_queue,
        halt_recv,
        report_queue,
        report_recv,
    };

    'main: loop {
        let listener = listener()?;

        for stream in listener.incoming().filter_map(Result::ok) {
            let request = match recv(&stream) {
                Ok(request) => request,
                Err(error) => {
                    send_text_error(&stream, error);
                    continue;
                }
            };

            free_halted_timers(&mut inner);

            match request {
                Request::Add { .. } => {
                    handle_add(request, &stream, &mut inner);
                }
                Request::Pause { .. }
                | Request::Halt { .. }
                | Request::Resume { .. } => {
                    handle_cmd(request, &stream, &mut inner)
                }
                Request::Report => handle_report(&stream, &inner),
                Request::Quit => {
                    handle_quit(&stream, inner);
                    break 'main;
                }
            }
        }
    }
    Ok(())
}

fn free_halted_timers(inner: &mut Inner) {
    loop {
        match inner.halt_recv.try_recv() {
            Ok(name) => {
                let removed = inner.timers.remove(&name);
                debug_assert!(removed.is_some())
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => unreachable!(),
        }
    }
}

fn handle_add<S: Write + Copy>(request: Request, stream: S, inner: &mut Inner) {
    let (name, duration, step) = match request {
        Request::Add {
            name,
            duration,
            step,
        } => (name, duration, step),
        _ => unreachable!(),
    };

    if duration.as_secs() == 0 || step.as_secs() == 0 {
        send_error(stream, Error::InvalidDuration);
    } else {
        let name = Arc::new(name);
        if inner.timers.get(&name).is_none() {
            let timer = Timer::spawn(
                Arc::clone(&name),
                duration,
                step,
                inner.updater.queue.clone(),
                inner.halt_queue.clone(),
                inner.report_queue.clone(),
            );
            inner.timers.insert(name, timer);
            send_ok(stream);
        } else {
            send_error(stream, Error::NameExists);
        }
    }
}

fn handle_cmd<S: Write + Copy>(request: Request, stream: S, inner: &mut Inner) {
    let (name, cmd): (_, fn(&Timer)) = match request {
        Request::Pause { name } => (name, Timer::pause),
        Request::Halt { name } => (name, Timer::halt),
        Request::Resume { name } => (name, Timer::resume),
        _ => unreachable!(),
    };

    if let Some(timer) = inner.timers.get(&Arc::new(name)) {
        cmd(timer);
        send_ok(stream);
    } else {
        send_error(stream, Error::NameNotExists);
    }
}

fn handle_report<S: Write + Copy>(stream: S, inner: &Inner) {
    inner.timers.values().for_each(Timer::report);
    let report = inner
        .report_recv
        .iter()
        .take(inner.timers.len())
        .filter(|usnapshot| !usnapshot.state.is_halted())
        .map(s::Snapshot::from)
        .collect();

    send_report(stream, report);
}

fn handle_quit<S: Write + Copy>(stream: S, inner: Inner) {
    let timers = inner.timers;
    timers.values().for_each(Timer::halt);
    timers.values().for_each(Timer::confirm_halt);
    timers.into_iter().for_each(|(_, timer)| Timer::join(timer));

    inner.updater.quit();
    inner.updater.join();
    send_ok(stream);
}

fn send<S: Write + Copy>(stream: S, response: Response) {
    if let Err(error) = serde_json::to_writer(stream, &response) {
        eprintln!("{}", error);
    }
}

fn send_ok<S: Write + Copy>(stream: S) {
    send(stream, Ok(None))
}

fn send_report<S: Write + Copy>(stream: S, report: HashSet<s::Snapshot>) {
    send(stream, Ok(Some(report)))
}

fn send_error<S: Write + Copy>(stream: S, error: Error) {
    send(stream, Err(error))
}

fn send_text_error<D: Display, S: Write + Copy>(stream: S, error: D) {
    send_error(
        stream,
        Error::Generic {
            message: error.to_string(),
        },
    )
}

fn recv<S: Read + Copy>(stream: S) -> serde_json::Result<Request> {
    serde_json::from_reader(stream)
}

fn daemonize() -> Result<(), Box<dyn std::error::Error>> {
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

impl From<u::Snapshot> for s::Snapshot {
    fn from(source: u::Snapshot) -> Self {
        s::Snapshot {
            name: source.name.to_string(),
            duration: source.duration,
            elapsed: source.elapsed,
            state: source.state,
        }
    }
}
