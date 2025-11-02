use std::path::Path;

use anyhow::Result;
use tokio::net::UnixSocket;

use crate::firecracker::FirecrackerStream;

/// Structure for initializing a Unix socket for communication with a Firecracker
pub struct FirecrackerSocket {
    socket: UnixSocket,
}

impl FirecrackerSocket {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: UnixSocket::new_stream()?,
        })
    }

    /// Creates a Unix stream for communicating with the Firecracker via a specified path
    pub async fn connect<P: AsRef<Path>>(self, path: P) -> Result<FirecrackerStream> {
        let stream = self.socket.connect(&path).await?;
        Ok(FirecrackerStream::new(stream))
    }
}
