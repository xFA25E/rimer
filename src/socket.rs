use crate::request::Request;

use std::{
    fs::DirBuilder,
    io,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

pub fn listener() -> io::Result<UnixListener> {
    let sock = path()?;

    if sock.exists() {
        if let Ok(stream) = UnixStream::connect(&sock) {
            if let Err(e) = serde_json::to_writer(&stream, &Request::Quit) {
                eprintln!("{}", e);
            }
        }
        std::fs::remove_file(&sock)?;
    }

    UnixListener::bind(sock)
}

pub fn path() -> std::io::Result<PathBuf> {
    let mut p = dirs::runtime_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    DirBuilder::new().recursive(true).create(&p)?;
    p.push("rimer.socket");
    Ok(p)
}
