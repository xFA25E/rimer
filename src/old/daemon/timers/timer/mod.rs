mod inner;

pub use inner::TimerUpdater;

use self::inner::{InnerTimer, Message, TimerKind, TimerStateFlag};

use std::{
    fmt::{self, Display},
    sync::{
        atomic::{AtomicU64, Ordering::Relaxed},
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
};

pub struct Timer {
    tx: Sender<Message>,
    name: Arc<String>,
    elapsed: Arc<AtomicU64>,
    kind: TimerKind,
    dur: u64,
    state: Arc<TimerStateFlag>,
}

macro_rules! timer_sender {
    ($func:ident, $msg:expr) => {
        pub fn $func(&self) {
            self.send($msg);
        }
    };
}

impl Timer {
    pub fn name(&self) -> &str {
        &*self.name
    }

    pub fn new(
        name: String,
        dur: u64,
        step: u64,
        upd: Arc<TimerUpdater>,
        last_update: Arc<Mutex<u64>>,
        kind: TimerKind,
    ) -> Self {
        debug_assert!(step != 0, "Step can't be zero!");

        let name = Arc::new(name);
        let elapsed = Arc::new(AtomicU64::new(0));
        let state = Arc::new(TimerStateFlag::new());
        let (tx, rx) = mpsc::channel();

        thread::Builder::new()
            .name(name.to_string())
            .spawn({
                let t = InnerTimer {
                    rx,
                    name: Arc::clone(&name),
                    dur,
                    step,
                    elapsed: Arc::clone(&elapsed),
                    state: Arc::clone(&state),
                    upd,
                    last_update,
                    kind,
                };
                move || t.spawn()
            })
            .unwrap();

        Self {
            tx,
            name,
            dur,
            elapsed,
            state,
            kind,
        }
    }

    pub fn countdown(
        name: String,
        dur: u64,
        step: u64,
        upd: Arc<TimerUpdater>,
        last_update: Arc<Mutex<u64>>,
    ) -> Self {
        Self::new(name, dur, step, upd, last_update, TimerKind::Countdown)
    }

    pub fn stopwatch(
        name: String,
        step: u64,
        upd: Arc<TimerUpdater>,
        last_update: Arc<Mutex<u64>>,
    ) -> Self {
        Self::new(
            name,
            u64::max_value(),
            step,
            upd,
            last_update,
            TimerKind::Stopwatch,
        )
    }

    fn send(&self, m: Message) {
        if !self.state.is_halted() {
            if let Err(e) = self.tx.send(m) {
                eprintln!("Could not send message: {}", e);
            }
        }
    }

    pub fn is_halted(&self) -> bool {
        self.state.is_halted()
    }

    timer_sender!(pause, Message::Pause);
    timer_sender!(resume, Message::Resume);
    timer_sender!(halt, Message::Halt);
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.halt();
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.name,
            self.kind,
            self.elapsed.load(Relaxed),
            self.dur,
            self.state.get()
        )
    }
}
