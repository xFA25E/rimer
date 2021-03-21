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

enum Message {
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
    arg: Arc<String>,
}

impl Timer {
    pub fn spawn(
        name: Arc<String>,
        duration: Duration,
        step: Duration,
        update_queue: Sender<u::Message>,
        halt_queue: Sender<Arc<String>>,
        report_queue: Sender<Snapshot>,
        arg: Arc<String>,
    ) -> Self {
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
                    arg,
                })
            })
            .unwrap();

        Self { handle, queue }
    }

    pub fn pause(&self) {
        self.queue.send(Message::Pause).unwrap();
    }

    pub fn resume(&self) {
        self.queue.send(Message::Resume).unwrap();
    }

    pub fn halt(&self) {
        self.queue.send(Message::Halt).unwrap();
    }

    pub fn report(&self) {
        self.queue.send(Message::Report).unwrap();
    }

    pub fn confirm_halt(&self) {
        self.queue.send(Message::ConfirmHalt).unwrap();
    }

    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

fn run(mut inner: Inner) {
    if inner.step > inner.duration {
        inner.step = inner.duration;
    }

    while inner.elapsed < inner.duration {
        inner.send_update();

        if let Message::Halt = inner.wait_message() {
            break;
        }

        inner.update_elapsed();
        inner.update_step();
    }

    inner.state = State::Halted;

    inner.send_update();
    inner.send_halt();

    for message in &inner.receiver {
        match message {
            Message::ConfirmHalt => break,
            Message::Report => inner.send_report(),
            _ => (),
        }
    }
}

impl Inner {
    fn wait_message(&mut self) -> Message {
        match self.receiver.recv_timeout(self.step) {
            Ok(Message::Pause) => {
                self.state = State::Paused;
                self.send_update();
                if let Message::Halt = self.wait_resume() {
                    return Message::Halt;
                }
            }
            Ok(Message::Halt) => {
                self.update_elapsed();
                return Message::Halt;
            }
            Ok(Message::Report) => {
                self.update_elapsed();
                self.send_report();
            }
            Err(RecvTimeoutError::Timeout) | Ok(Message::Resume) => (),
            Ok(Message::ConfirmHalt) | Err(RecvTimeoutError::Disconnected) => {
                unreachable!()
            }
        }
        Message::Resume
    }

    fn wait_resume(&mut self) -> Message {
        self.update_elapsed();
        loop {
            match self.receiver.recv() {
                Ok(Message::Resume) => {
                    self.state = State::Running;
                    self.start_time = SystemTime::now().checked_sub(self.elapsed).unwrap();
                    break;
                }
                Ok(Message::Pause) => (),
                Ok(Message::Report) => {
                    self.send_report();
                }
                Ok(Message::Halt) => {
                    return Message::Halt;
                }
                Ok(Message::ConfirmHalt) | Err(_) => unreachable!(),
            }
        }
        Message::Resume
    }

    fn make_snapshot(&self) -> Snapshot {
        Snapshot {
            name: Arc::clone(&self.name),
            duration: self.duration,
            elapsed: self.elapsed,
            state: self.state,
            arg: Arc::clone(&self.arg),
        }
    }

    fn send_update(&self) {
        let snapshot = self.make_snapshot();
        u::update(&self.update_queue, snapshot);
    }

    fn send_halt(&self) {
        self.halt_queue.send(Arc::clone(&self.name)).unwrap();
    }

    fn send_report(&self) {
        let snapshot = self.make_snapshot();
        self.report_queue.send(snapshot).unwrap();
    }

    fn update_step(&mut self) {
        let Self {
            elapsed,
            duration,
            step,
            ..
        } = self;

        if elapsed < duration && *duration < *elapsed + *step {
            *step = *duration - *elapsed;
        }
    }

    fn update_elapsed(&mut self) {
        if let Ok(new_elapsed) = self.start_time.elapsed() {
            self.elapsed = new_elapsed;
        } else {
            self.start_time = SystemTime::now().checked_sub(self.elapsed).unwrap();
        }
    }
}
