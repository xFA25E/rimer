use super::{request::Request, response::Response, socket};
use std::{io::stdout, net::Shutdown};

pub fn run(request: Request) -> std::io::Result<()> {
    let stream = socket::stream()?;
    serde_json::to_writer(&stream, &request)?;
    stream.shutdown(Shutdown::Write)?;

    let response: Response = serde_json::from_reader(&stream)?;
    match response {
        Ok(Some(snapshots)) => {
            if let Request::Report { json: true } = request {
                let mut lock = stdout();
                serde_json::to_writer(&mut lock, &snapshots)?;
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
