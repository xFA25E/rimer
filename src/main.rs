mod config;
mod daemon;
mod remote;
mod request;
mod socket;

pub use daemon::TimerUpdater;

use config::Config;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Config::new() {
        Config::Daemon(updater) => self::daemon::run(updater)?,
        Config::Remote(request) => self::remote::run(request)?,
    }

    Ok(())
}
