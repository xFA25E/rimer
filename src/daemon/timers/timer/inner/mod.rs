mod state;
mod updater;

pub use self::{state::TimerStateFlag, updater::TimerUpdater};

use self::updater::TimerSnapshot;

use std::{
    fmt::{self, Display},
    sync::{
        atomic::{AtomicU64, Ordering::Relaxed},
        mpsc::{Receiver, RecvError, TryRecvError},
        Arc, Mutex,
    },
    time::Duration,
};

pub struct InnerTimer {
    pub rx: Receiver<Message>,
    pub name: Arc<String>,
    pub dur: u64,
    pub step: u64,
    pub elapsed: Arc<AtomicU64>,
    pub state: Arc<TimerStateFlag>,
    pub upd: Arc<TimerUpdater>,
    pub last_update: Arc<Mutex<u64>>,
    pub kind: TimerKind,
}

impl InnerTimer {
    pub fn spawn(self) {
        self.upd.exec(self.snapshot());

        std::thread::sleep(ONE_SECOND);

        for (update, n) in (1..=self.dur).map(|n| (n % self.step == 0, n)) {
            self.elapsed.store(n, Relaxed);

            if update && self.should_update() {
                self.upd.exec(self.snapshot());
            }

            if let Event::Halt = self.listen_event() {
                break;
            }

            std::thread::sleep(ONE_SECOND);
        }

        *self.last_update.lock().unwrap() = 0;
        self.state.halt();
        self.upd.exec(self.snapshot());
    }

    fn should_update(&self) -> bool {
        let mut update = false;
        let left = self.dur - self.elapsed.load(Relaxed);

        let mut lu_lock = self.last_update.lock().unwrap();
        if left < *lu_lock || *lu_lock == 0 {
            *lu_lock = left;
            update = true;
        }

        update
    }

    fn listen_event(&self) -> Event {
        match self.rx.try_recv() {
            Ok(Message::Pause) => {
                *self.last_update.lock().unwrap() = 0;
                self.state.pause();
                self.upd.exec(self.snapshot());
                self.wait_pause()
            }

            Ok(Message::Halt) | Err(TryRecvError::Disconnected) => Event::Halt,

            _ => Event::Other,
        }
    }

    fn wait_pause(&self) -> Event {
        loop {
            match self.rx.recv() {
                Ok(Message::Resume) => {
                    self.state.resume();
                    self.upd.exec(self.snapshot());
                    return Event::Other;
                }

                Ok(Message::Halt) | Err(RecvError) => return Event::Halt,

                _ => continue,
            }
        }
    }

    fn snapshot(&self) -> TimerSnapshot {
        TimerSnapshot {
            name: Arc::clone(&self.name),
            kind: self.kind,
            elapsed: self.elapsed.load(Relaxed),
            dur: self.dur,
            state: self.state.get(),
        }
    }
}

enum Event {
    Other,
    Halt,
}

pub enum Message {
    Pause,
    Resume,
    Halt,
}

#[derive(Clone, Copy)]
pub enum TimerKind {
    Countdown,
    Stopwatch,
}

impl Display for TimerKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Countdown => write!(f, "countdown"),
            Self::Stopwatch => write!(f, "stopwatch"),
        }
    }
}

const ONE_SECOND: Duration = Duration::from_secs(1);
