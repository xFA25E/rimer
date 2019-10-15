use crate::{request::Request, socket};

use std::{io::Read, net::Shutdown, os::unix::net::UnixStream};

pub fn run(request: Request) -> std::io::Result<()> {
    let mut stream = UnixStream::connect(socket::path()?)?;
    serde_json::to_writer(&stream, &request)?;

    if let Request::Report = request {
        stream.shutdown(Shutdown::Write)?;
        let mut result = String::new();
        stream.read_to_string(&mut result)?;
        print!("{}", result);
    }
    Ok(())
}
