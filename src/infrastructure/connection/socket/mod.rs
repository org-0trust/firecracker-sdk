use std::path::Path;

use anyhow::Result;
use tokio::net::UnixSocket;

use crate::infrastructure::connection::stream::Stream;

/// Structure for initializing a Unix socket for communication with a Firecracker
pub struct Socket {
    socket: UnixSocket,
}

impl Socket {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: UnixSocket::new_stream()?,
        })
    }

    /// Creates a Unix stream for communicating with the Firecracker via a specified path
    pub async fn connect<P: AsRef<Path>>(self, path: P) -> Result<Stream> {
        let stream = self.socket.connect(&path).await?;
        Ok(Stream::new(stream))
    }
}
