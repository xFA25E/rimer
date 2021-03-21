use super::{request::Request, response::Response, socket};
use std::{io, net::Shutdown};

pub fn run(request: Request) -> std::io::Result<()> {
    let stream = socket::stream()?;
    serde_json::to_writer(&stream, &request)?;
    stream.shutdown(Shutdown::Write)?;

    let response: Response = serde_json::from_reader(&stream)?;
    match response {
        Ok(Some(snapshots)) => {
            if let Request::Report { json: true } = request {
                let lock = io::stdout();
                serde_json::to_writer(&lock, &snapshots)?;
            } else {
                for snapshot in snapshots {
                    println!("{}", snapshot);
                }
            }
        }
        Ok(None) => (),
        Err(error) => eprintln!("{}", error),
    }
    Ok(())
}
