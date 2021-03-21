use super::request::Request;
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

pub fn stream() -> io::Result<UnixStream> {
    UnixStream::connect(path()?)
}

pub fn path() -> std::io::Result<PathBuf> {
    let mut p = dirs::runtime_dir().unwrap_or_else(|| {
        eprintln!("XDG_RUNTIME_DIR is undefined! Using /tmp.");
        PathBuf::from("/tmp")
    });
    DirBuilder::new().recursive(true).create(&p)?;
    p.push("rimer.socket");
    Ok(p)
}
