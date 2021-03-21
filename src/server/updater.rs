use crate::state::State;
use std::{
    process::Command,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

pub struct Updater {
    pub handle: thread::JoinHandle<()>,
    pub queue: Sender<Message>,
}

pub enum Message {
    Update { snapshot: Snapshot },
    Quit,
}

pub struct Snapshot {
    pub name: Arc<String>,
    pub duration: Duration,
    pub elapsed: Duration,
    pub state: State,
    pub arg: Arc<String>,
}

impl Updater {
    pub fn spawn(command: String) -> Self {
        let (queue, receiver) = channel();

        let handle = thread::Builder::new()
            .name("updater".into())
            .spawn(|| run(command, receiver))
            .unwrap();

        Updater { queue, handle }
    }

    pub fn quit(&self) {
        self.queue.send(Message::Quit).unwrap();
    }

    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

pub fn update(queue: &Sender<Message>, snapshot: Snapshot) {
    queue.send(Message::Update { snapshot }).unwrap();
}

fn run(command: String, receiver: Receiver<Message>) {
    let mut last_update = std::time::Duration::from_secs(0);

    for update in receiver {
        match update {
            Message::Update { snapshot } => handle_update(snapshot, &mut last_update, &command),
            Message::Quit => break,
        }
    }
}

fn handle_update(s: Snapshot, last_update: &mut Duration, command: &str) {
    let remaining = if s.duration > s.elapsed {
        s.duration - s.elapsed
    } else {
        Duration::from_secs(0)
    };

    if s.elapsed.as_secs() == 0
        || last_update.as_secs() == 0
        || remaining <= *last_update
        || s.state.is_halted()
        || s.state.is_paused()
    {
        *last_update = if s.state.is_paused() {
            Duration::from_secs(0)
        } else {
            remaining
        };

        if let Err(error) = Command::new(command)
            .arg(&*s.name)
            .arg(s.elapsed.as_secs().to_string())
            .arg(s.duration.as_secs().to_string())
            .arg(s.state.to_string())
            .arg(&*s.arg)
            .status()
        {
            eprintln!("{}", error);
        };
    }
}
