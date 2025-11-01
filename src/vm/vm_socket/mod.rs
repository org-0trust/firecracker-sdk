use std::path::Path;

use anyhow::Result;
use tokio::net::UnixSocket;

use crate::vm::VM;

pub struct VMSocket {
    socket: UnixSocket,
}

impl VMSocket {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: UnixSocket::new_stream()?,
        })
    }

    pub async fn connect<P: AsRef<Path>>(self, path: P) -> Result<VM> {
        let stream = self.socket.connect(&path).await?;
        Ok(VM::new(stream))
    }
}
