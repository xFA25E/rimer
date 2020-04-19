use super::updater::{self as u, Snapshot};
use crate::state::State;
use std::{
    sync::{
        mpsc::{channel, Receiver, RecvTimeoutError, Sender},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

pub struct Timer {
    handle: thread::JoinHandle<()>,
    queue: Sender<Message>,
}

pub enum Message {
    Pause,
    Resume,
    Halt,
    Report,
    ConfirmHalt,
}

struct Inner {
    name: Arc<String>,
    duration: Duration,
    step: Duration,
    receiver: Receiver<Message>,
    update_queue: Sender<u::Message>,
    halt_queue: Sender<Arc<String>>,
    report_queue: Sender<Snapshot>,
    start_time: SystemTime,
    state: State,
    elapsed: Duration,
}

pub fn spawn(
    name: Arc<String>,
    duration: Duration,
    step: Duration,
    update_queue: Sender<u::Message>,
    halt_queue: Sender<Arc<String>>,
    report_queue: Sender<Snapshot>,
) -> Timer {
    debug_assert_ne!(duration.as_secs(), 0);
    debug_assert_ne!(step.as_secs(), 0);

    let (queue, receiver) = channel();

    let handle = thread::Builder::new()
        .name("timer ".to_string() + &name)
        .spawn(move || {
            run(Inner {
                name,
                duration,
                step,
                receiver,
                update_queue,
                halt_queue,
                report_queue,
                start_time: SystemTime::now(),
                state: State::Running,
                elapsed: Duration::from_secs(0),
            })
        })
        .unwrap();

    Timer { handle, queue }
}

pub fn pause(timer: &Timer) {
    timer.queue.send(Message::Pause).unwrap();
}

pub fn resume(timer: &Timer) {
    timer.queue.send(Message::Resume).unwrap();
}

pub fn halt(timer: &Timer) {
    timer.queue.send(Message::Halt).unwrap();
}

pub fn report(timer: &Timer) {
    timer.queue.send(Message::Report).unwrap();
}

pub fn confirm_halt(timer: &Timer) {
    timer.queue.send(Message::ConfirmHalt).unwrap();
}

pub fn join(timer: Timer) {
    timer.handle.join().unwrap();
}

fn run(mut inner: Inner) {
    if inner.step > inner.duration {
        inner.step = inner.duration;
    }

    while inner.elapsed < inner.duration {
        send_update(&inner);

        if let Message::Halt = wait_message(&mut inner) {
            break;
        }

        update_elapsed(&mut inner);
        update_step(&mut inner);
    }

    inner.state = State::Halted;

    send_update(&inner);
    send_halt(&inner);

    for message in &inner.receiver {
        match message {
            Message::ConfirmHalt => break,
            Message::Report => send_report(&inner),
            _ => (),
        }
    }
}

fn wait_message(inner: &mut Inner) -> Message {
    match inner.receiver.recv_timeout(inner.step) {
        Ok(Message::Pause) => {
            inner.state = State::Paused;
            send_update(inner);
            if let Message::Halt = wait_resume(inner) {
                return Message::Halt;
            }
        }
        Ok(Message::Halt) => {
            update_elapsed(inner);
            return Message::Halt;
        }
        Ok(Message::Report) => {
            update_elapsed(inner);
            send_report(inner);
        }
        Err(RecvTimeoutError::Timeout) | Ok(Message::Resume) => (),
        Ok(Message::ConfirmHalt) | Err(RecvTimeoutError::Disconnected) => {
            unreachable!()
        }
    }
    Message::Resume
}

fn wait_resume(inner: &mut Inner) -> Message {
    loop {
        match inner.receiver.recv() {
            Ok(Message::Resume) => {
                inner.state = State::Running;
                inner.start_time =
                    SystemTime::now().checked_sub(inner.elapsed).unwrap();
                break;
            }
            Ok(Message::Pause) => (),
            Ok(Message::Report) => {
                update_elapsed(inner);
                send_report(inner);
            }
            Ok(Message::Halt) => {
                update_elapsed(inner);
                return Message::Halt;
            }
            Ok(Message::ConfirmHalt) | Err(_) => unreachable!(),
        }
    }
    Message::Resume
}

fn make_snapshot(inner: &Inner) -> Snapshot {
    Snapshot {
        name: Arc::clone(&inner.name),
        duration: inner.duration,
        elapsed: inner.elapsed,
        state: inner.state,
    }
}

fn send_update(inner: &Inner) {
    let snapshot = make_snapshot(inner);
    u::update(&inner.update_queue, snapshot);
}

fn send_halt(inner: &Inner) {
    inner.halt_queue.send(Arc::clone(&inner.name)).unwrap();
}

fn send_report(inner: &Inner) {
    let snapshot = make_snapshot(inner);
    inner.report_queue.send(snapshot).unwrap();
}

fn update_step(inner: &mut Inner) {
    let Inner {
        elapsed,
        duration,
        step,
        ..
    } = inner;

    if elapsed < duration && *duration < *elapsed + *step {
        *step = *duration - *elapsed;
    }
}

fn update_elapsed(inner: &mut Inner) {
    if let Ok(new_elapsed) = inner.start_time.elapsed() {
        inner.elapsed = new_elapsed;
    } else {
        inner.start_time =
            SystemTime::now().checked_sub(inner.elapsed).unwrap();
    }
}

#[cfg(test)]
mod timer {
    use super::*;

    #[test]
    fn test_timer() {
        let (update_queue, update_recv) = channel();
        let (halt_queue, halt_recv) = channel();
        let (report_queue, report_recv) = channel();

        let timer = spawn(
            Arc::new("hello".into()),
            Duration::from_secs(30),
            Duration::from_secs(6),
            update_queue.clone(),
            halt_queue.clone(),
            report_queue.clone(),
        );

        let timer2 = spawn(
            Arc::new("hello bong".into()),
            Duration::from_secs(30),
            Duration::from_secs(5),
            update_queue,
            halt_queue,
            report_queue,
        );

        let start = SystemTime::now();
        for message in update_recv {
            match message {
                u::Message::Update { snapshot } => println!(
                    "{} {:?} {:?} {} {:?}",
                    snapshot.name,
                    snapshot.duration,
                    snapshot.elapsed,
                    snapshot.state,
                    start.elapsed().unwrap()
                ),
                _ => (),
            }
        }
    }
}
