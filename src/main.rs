mod client;
mod config;
mod request;
mod response;
mod server;
mod snapshot;
mod socket;
mod state;

use config::Config;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Config::new() {
        Config::Server { callback } => server::run(callback)?,
        Config::Client { request } => client::run(request)?,
    }
    Ok(())
}
