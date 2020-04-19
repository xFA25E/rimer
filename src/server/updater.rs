use crate::state::{is_halted, is_paused, State};
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
}

pub fn spawn(command: String) -> Updater {
    let (queue, receiver) = channel();

    let handle = thread::Builder::new()
        .name("updater".into())
        .spawn(|| run(command, receiver))
        .unwrap();

    Updater { queue, handle }
}

pub fn update(queue: &Sender<Message>, snapshot: Snapshot) {
    queue.send(Message::Update { snapshot }).unwrap();
}

pub fn quit(updater: &Updater) {
    updater.queue.send(Message::Quit).unwrap();
}

pub fn join(updater: Updater) {
    updater.handle.join().unwrap();
}

fn run(command: String, receiver: Receiver<Message>) {
    let mut last_update = std::time::Duration::from_secs(0);

    for update in receiver {
        match update {
            Message::Update { snapshot } => {
                handle_update(snapshot, &mut last_update, &command)
            }
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
        || is_halted(s.state)
        || is_paused(s.state)
    {
        if is_paused(s.state) {
            *last_update = Duration::from_secs(0);
        } else {
            *last_update = remaining;
        }

        if let Err(error) = Command::new(command)
            .arg(&*s.name)
            .arg(s.elapsed.as_secs().to_string())
            .arg(s.duration.as_secs().to_string())
            .arg(s.state.to_string())
            .status()
        {
            eprintln!("{}", error);
        };
    }
}
